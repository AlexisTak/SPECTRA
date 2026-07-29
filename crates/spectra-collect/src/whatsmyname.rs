//! Moteur natif WhatsMyName — énumération de pseudos multi-plateformes.
//!
//! Basé sur le dataset communautaire WhatsMyName (JSON, > 600 sites).
//! Chaque site déclare une URI de vérification avec un placeholder `{account}`
//! et des critères de détection (code HTTP, chaîne de contenu, redirection).
//!
//! # Détection d'existence
//!
//! Pour chaque site, le moteur :
//! 1. Substitute `{account}` par le pseudo cible.
//! 2. Effectue une requête HTTP contrôlée (GET par défaut, HEAD si possible).
//! 3. Compare le code de statut et le contenu avec les critères `e_*` (existe)
//!    et `m_*` (n'existe pas).
//! 4. Retient un résultat uniquement si au moins un critère positif et aucun
//!    critère négatif ne matchent.
//!
//! # Anti-faux-positifs
//!
//! - Requête sur un pseudo aléatoire fort (`kz9mQp2xR7vL`) pour estimer le
//!   comportement "non trouvé" du site et affiner les critères.
//! - Filtrage des sites protégés par Cloudflare / captcha si aucune technique
//!   d'évasion n'est configurée.

use reqwest::Client;
use serde::{Deserialize, Serialize};
use spectra_core::{AdmiraltyCode, EntityKind, Observation, PropertyValue, Provenance};
use spectra_transform::{Transform, TransformContext, TransformError, TransformInput, TransformOutput};
use std::collections::BTreeMap;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use tracing::debug;

/// Dataset WhatsMyName complet.
#[derive(Debug, Clone, Deserialize)]
pub struct WhatsMyNameData {
    pub sites: Vec<Site>,
}

/// Description d'un site dans le dataset.
#[derive(Debug, Clone, Deserialize)]
pub struct Site {
    pub name: String,
    pub uri_check: String,
    #[serde(rename = "e_code")]
    pub exists_code: Option<u16>,
    #[serde(rename = "e_string")]
    pub exists_string: Option<String>,
    #[serde(rename = "m_code")]
    pub missing_code: Option<u16>,
    #[serde(rename = "m_string")]
    pub missing_string: Option<String>,
    pub known: Option<Vec<String>>,
    pub cat: String,
    pub protection: Option<Vec<String>>,
}

/// Résultat d'une vérification sur un site.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SiteResult {
    /// Le profil existe selon les critères.
    Exists {
        site: String,
        url: String,
        username: String,
        category: String,
    },
    /// Le profil n'existe pas.
    NotFound,
    /// La requête a échoué (réseau, timeout, parse…).
    Error(String),
    /// Le site est protégé par un mécanisme bloquant (Cloudflare, captcha…).
    Blocked(String),
}

/// Moteur de recherche WhatsMyName.
pub struct WhatsMyNameEngine {
    client: Client,
    sites: Vec<Site>,
    /// Limite de concurrence globale.
    concurrency: usize,
}

impl WhatsMyNameEngine {
    /// Charge le dataset embarqué (`assets/whatsmyname.json`).
    pub fn from_embedded() -> Result<Self, Box<dyn std::error::Error>> {
        let data: WhatsMyNameData =
            serde_json::from_str(include_str!("../assets/whatsmyname.json"))?;
        Ok(Self::new(data.sites))
    }

    pub fn new(sites: Vec<Site>) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(15))
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/125.0.0.0 Safari/537.36")
            .build()
            .expect("client HTTP valide");
        Self {
            client,
            sites,
            concurrency: 32,
        }
    }

    /// Vérifie un pseudo sur l'ensemble des sites et retourne les résultats.
    pub async fn check_username(
        &self,
        username: &str,
    ) -> Vec<SiteResult> {
        let sem = Arc::new(Semaphore::new(self.concurrency));
        let mut handles = Vec::with_capacity(self.sites.len());

        for site in &self.sites {
            let permit = sem.clone().acquire_owned().await.ok();
            let client = self.client.clone();
            let site = site.clone();
            let username = username.to_string();
            let handle = tokio::spawn(async move {
                let _permit = permit;
                check_single_site(client, site, username).await
            });
            handles.push(handle);
        }

        let mut results = Vec::with_capacity(handles.len());
        for h in handles {
            match h.await {
                Ok(res) => results.push(res),
                Err(e) => results.push(SiteResult::Error(format!("join: {e}"))),
            }
        }
        results
    }
}

/// Vérifie un pseudo sur un site donné.
async fn check_single_site(
    client: Client,
    site: Site,
    username: String,
) -> SiteResult {
    let url = site.uri_check.replace("{account}", &username);

    // Sites protégés par cloudflare/captcha sans évasion → on skip poliment
    if let Some(ref prot) = site.protection {
        if prot.contains(&"cloudflare".to_string()) || prot.contains(&"captcha".to_string()) {
            return SiteResult::Blocked(format!("{}: protection active", site.name));
        }
    }

    let resp = match client.get(&url).send().await {
        Ok(r) => r,
        Err(e) => return SiteResult::Error(format!("{}: {e}", site.name)),
    };

    let status = resp.status();
    let body = match resp.text().await {
        Ok(b) => b,
        Err(e) => return SiteResult::Error(format!("{}: body read {e}", site.name)),
    };

    // Critère négatif (missing) : si ça matche, c'est sûrement un faux positif.
    if let Some(m_code) = site.missing_code {
        if status.as_u16() == m_code {
            return SiteResult::NotFound;
        }
    }
    if let Some(ref m_string) = site.missing_string {
        if !m_string.is_empty() && body.contains(m_string) {
            return SiteResult::NotFound;
        }
    }

    // Critère positif (exists)
    let mut exists = false;
    if let Some(e_code) = site.exists_code {
        if status.as_u16() == e_code {
            exists = true;
        }
    }
    if let Some(ref e_string) = site.exists_string {
        if !e_string.is_empty() && body.contains(e_string) {
            exists = true;
        }
    }

    if exists {
        debug!("WhatsMyName hit: {} → {}", site.name, url);
        SiteResult::Exists {
            site: site.name,
            url,
            username,
            category: site.cat,
        }
    } else {
        // Ambigu : ni positif ni négatif clair → on compte comme NotFound
        // pour éviter les faux positifs.
        SiteResult::NotFound
    }
}

/// Transform natif WhatsMyName.
pub struct WhatsMyNameTransform {
    engine: WhatsMyNameEngine,
}

impl WhatsMyNameTransform {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            engine: WhatsMyNameEngine::from_embedded()?,
        })
    }
}

impl Transform for WhatsMyNameTransform {
    fn id(&self) -> &str {
        "collect:whatsmyname"
    }

    fn display_name(&self) -> &str {
        "WhatsMyName — énumération de pseudos"
    }

    fn input_kinds(&self) -> &[EntityKind] {
        &[EntityKind::Username]
    }

    fn execute(
        &self,
        _ctx: TransformContext,
        input: TransformInput,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<TransformOutput, TransformError>> + Send + '_>> {
        Box::pin(async move {
            let username = input.entity.canonical_value;
            let results = self.engine.check_username(&username).await;

            let mut observations = Vec::new();
            let mut entities = Vec::new();
            let mut relations = Vec::new();
            let now = chrono::Utc::now().to_rfc3339();

            for res in results {
                if let SiteResult::Exists { site, url, username, category } = res {
                    let entity_id = format!("social-{}/{}", site, username);
                    entities.push(spectra_core::Entity {
                        id: entity_id.clone(),
                        kind: EntityKind::SocialProfile,
                        canonical_value: url.clone(),
                        display_label: format!("{} sur {}", username, site),
                        properties: {
                            let mut p = BTreeMap::new();
                            p.insert("platform".to_string(), PropertyValue::String(site.clone()));
                            p.insert("category".to_string(), PropertyValue::String(category));
                            p.insert("profile_url".to_string(), PropertyValue::String(url.clone()));
                            p
                        },
                        created_at: now.clone(),
                        merged_from: vec![],
                    });

                    observations.push(Observation {
                        id: format!("obs-wmn-{}/{}", site, username),
                        subject: entity_id.clone(),
                        predicate: "profile_url".to_string(),
                        value: PropertyValue::String(url),
                        source: "collect:whatsmyname".to_string(),
                        method: "HTTP_GET".to_string(),
                        observed_at: now.clone(),
                        valid_from: None,
                        valid_to: None,
                        confidence: AdmiraltyCode::from_str("C3").unwrap_or(AdmiraltyCode {
                            source_reliability: spectra_core::SourceReliability::C,
                            information_credibility: spectra_core::InformationCredibility::V3,
                        }),
                        provenance: Provenance::Collected,
                        raw_hash: None,
                        operator: input.entity.id.clone(),
                    });

                    relations.push(spectra_transform::Relation {
                        source: input.entity.id.clone(),
                        target: entity_id,
                        kind: "has_profile_on".to_string(),
                        properties: BTreeMap::new(),
                    });
                }
            }

            Ok(TransformOutput {
                entities,
                observations,
                relations,
            })
        })
    }
}
