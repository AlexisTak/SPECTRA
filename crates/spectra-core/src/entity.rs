//! Entités et observations.
//!
//! # Rôle
//!
//! Définit ce qu'est une entité (Personne, Domaine, Email, etc.) et surtout
//! une [`Observation`] : une propriété sourcée, datée, qualifiée. Le graphe
//! affiché est une *projection* des observations à un instant T, jamais un
//! stockage direct.
//!
//! # Invariants
//!
//! 1. **Aucune I/O.** Les horodatages sont fournis par l'appelant.
//! 2. **Aucune fusion automatique.** La résolution d'identité propose, un
//!    humain décide, et la fusion reste réversible.
//! 3. **Une propriété n'est jamais un fait nu.** C'est une observation datée,
//!    sourcée et qualifiée.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Identifiant unique d'entité. UUID v7 (ordonnables temporellement).
pub type EntityId = String;

/// Identifiant d'observation.
pub type ObservationId = String;

/// Provenance d'une observation.
///
/// C'est ce qui sépare un raisonnement d'un fait collecté. Toute sortie de LLM
/// est marquée `Inferred` et exclue des rapports par défaut.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum Provenance {
    /// Collecté depuis une source externe (HTTP, fichier, API).
    Collected,
    /// Asserté par un analyste (saisie manuelle).
    Asserted,
    /// Inféré par un modèle (IA, heuristique non déterministe).
    Inferred,
}

/// Niveau de confiance (échelle Admiralty A1–F6).
///
/// `A1` = source très fiable, information confirmée. `F6` = source
/// inconnue, information non vérifiée.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct AdmiraltyCode {
    /// Fiabilité de la source : A (très fiable) à F (inconnue).
    pub source_reliability: SourceReliability,
    /// Crédibilité de l'information : 1 (confirmée) à 6 (douteuse).
    pub information_credibility: InformationCredibility,
}

/// Fiabilité de la source (échelle Admiralty).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum SourceReliability {
    /// Source très fiable.
    A,
    /// Source fiable.
    B,
    /// Source assez fiable.
    C,
    /// Source pas toujours fiable.
    D,
    /// Source peu fiable.
    E,
    /// Source inconnue / non évaluée.
    F,
}

/// Crédibilité de l'information (échelle Admiralty).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum InformationCredibility {
    /// Confirmée par d'autres sources.
    #[serde(rename = "1")]
    V1 = 1,
    /// Probable.
    #[serde(rename = "2")]
    V2,
    /// Possible.
    #[serde(rename = "3")]
    V3,
    /// Douteuse.
    #[serde(rename = "4")]
    V4,
    /// Improbable.
    #[serde(rename = "5")]
    V5,
    /// Non vérifiée / à vérifier.
    #[serde(rename = "6")]
    V6,
}

impl AdmiraltyCode {
    /// Constructeur pratique : `"B3"` → `Some(AdmiraltyCode { source_reliability: B, information_credibility: V3 })`.
    pub fn from_str(s: &str) -> Option<Self> {
        if s.len() != 2 {
            return None;
        }
        let src = s.chars().next()?;
        let cred = s.chars().nth(1)?;
        let source_reliability = match src {
            'A' => Some(SourceReliability::A),
            'B' => Some(SourceReliability::B),
            'C' => Some(SourceReliability::C),
            'D' => Some(SourceReliability::D),
            'E' => Some(SourceReliability::E),
            'F' => Some(SourceReliability::F),
            _ => None,
        }?;
        let information_credibility = match cred {
            '1' => Some(InformationCredibility::V1),
            '2' => Some(InformationCredibility::V2),
            '3' => Some(InformationCredibility::V3),
            '4' => Some(InformationCredibility::V4),
            '5' => Some(InformationCredibility::V5),
            '6' => Some(InformationCredibility::V6),
            _ => None,
        }?;
        Some(Self {
            source_reliability,
            information_credibility,
        })
    }
}

/// Type d'entité. Extensible via des fichiers TOML (ontologie), mais le socle
/// en définit quelques-uns en dur pour démarrer.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum EntityKind {
    Person,
    Alias,
    EmailAddress,
    PhoneNumber,
    Username,
    SocialProfile,
    Domain,
    IpAddress,
    Netblock,
    Organization,
    Location,
    Document,
    Image,
    CryptoAddress,
    Device,
    Event,
    Vehicle,
}

/// Valeur de propriété.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum PropertyValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    DateTime(String),
    Json(serde_json::Value),
}

/// Une entité du graphe.
///
/// Une entité n'est **jamais** un fait nu : ses propriétés sont portées par des
/// [`Observation`]. Cette structure est une vue agrégée pratique pour
/// l'affichage, mais la source de vérité est le flux d'observations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub id: EntityId,
    pub kind: EntityKind,
    /// Valeur normalisée (email en minuscules, téléphone en E.164, etc.).
    pub canonical_value: String,
    /// Label affiché dans l'UI.
    pub display_label: String,
    /// Propriétés dérivées pour l'affichage. La vérité est dans les observations.
    pub properties: BTreeMap<String, PropertyValue>,
    pub created_at: String,
    /// Les fusions sont réversibles : on conserve la trace des entités absorbées.
    pub merged_from: Vec<EntityId>,
}

/// Une observation : une propriété sourcée, datée, qualifiée.
///
/// C'est le cœur du modèle probatoire. Sans observation, pas de traçabilité,
/// pas de confiance mesurée, pas de distinction entre fait collecté et
/// raisonnement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Observation {
    pub id: ObservationId,
    pub subject: EntityId,
    /// Prédicat : `"email"`, `"pseudo"`, `"avatar_url"`, `"bio"`, etc.
    pub predicate: String,
    pub value: PropertyValue,
    /// Source : nom du transform, identifiant de plugin, ou analyste.
    pub source: String,
    /// Méthode d'acquisition : `"HTTP_GET"`, `"WHOIS"`, `"MANUAL"`, `"LLM_NER"`.
    pub method: String,
    /// Quand SPECTRA a vu cette observation.
    pub observed_at: String,
    /// Quand c'était vrai dans le monde réel (optionnel).
    pub valid_from: Option<String>,
    /// Quand ça a cessé d'être vrai (optionnel).
    pub valid_to: Option<String>,
    /// Niveau de confiance (échelle Admiralty).
    pub confidence: AdmiraltyCode,
    pub provenance: Provenance,
    /// Hash du contenu brut archivé (optionnel).
    pub raw_hash: Option<String>,
    /// Analyste ou processus à l'origine de l'observation.
    pub operator: String,
}
