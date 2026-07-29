//! Commandes OSINT — pont entre Tauri et spectra-probe.

use crate::database::AppState;
use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use spectra_probe::{ProbeEngine, ProbeOutcome as SpectraOutcome};
use tauri::command;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SelectorKind {
    Username,
    Email,
    Phone,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProbeOutcome {
    Exists { url: Option<String> },
    Missing,
    Blocked { reason: String },
    Indeterminate { reason: String },
    Error { message: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeResult {
    pub site: String,
    pub outcome: ProbeOutcome,
    pub elapsed_ms: u64,
    /// Vrai si la règle de la sonde a échoué au contrôle anti-faux-positifs.
    pub degraded: bool,
}

/// Lance une campagne OSINT sur un sélecteur donné.
///
/// Utilise le moteur spectra-probe pour exécuter les sondes.
#[command]
pub async fn run_osint_campaign(
    state: tauri::State<'_, AppState>,
    selector: String,
    kind: SelectorKind,
) -> AppResult<Vec<ProbeResult>> {
    let storage_root = state.storage_root().to_path_buf();

    // Le chargement lit un fichier : il ne doit pas se faire en tenant le
    // verrou de la base, sinon toute autre commande est bloquée pendant l'I/O.
    let probes = load_probes(&storage_root)?;

    // Filtrer par type de sélecteur
    let filtered: Vec<_> = probes
        .into_iter()
        .filter(|p| {
            matches!(
                (p.selector_kind.clone(), &kind),
                (spectra_probe::SelectorKind::Username, SelectorKind::Username)
                    | (spectra_probe::SelectorKind::Email, SelectorKind::Email)
                    | (spectra_probe::SelectorKind::Phone, SelectorKind::Phone)
            )
        })
        .collect();

    if filtered.is_empty() {
        // Renvoyer une liste vide plutôt que des résultats fabriqués : un outil
        // d'enquête ne doit jamais présenter de faux « comptes trouvés »
        // (`audit.md`, P3-10). L'absence de sonde est un état, pas un résultat.
        return Ok(Vec::new());
    }

    // User-Agent honnête et identifiable, sans rotation ni déguisement :
    // SPECTRA est un outil professionnel qui s'annonce.
    let engine = ProbeEngine::new(
        concat!(
            "Cekarna/",
            env!("CARGO_PKG_VERSION"),
            " (outil d'investigation OSINT; +https://github.com/cekarna/enquetes)"
        )
        .to_string(),
        50,
    );

    // Bilan de santé avant la recherche : une sonde qui répond « existe » sur
    // un pseudo aléatoire ne peut rien affirmer sur le vrai sélecteur.
    let degraded = degraded_probes(&engine, &filtered).await;

    let cancel = tokio_util::sync::CancellationToken::new();
    let results = engine.run_campaign(&selector, &filtered, cancel).await;

    let converted: Vec<ProbeResult> = results
        .into_iter()
        .map(|r| {
            let degradation = degraded.get(&r.probe_id.0).cloned();
            present_result(r, degradation)
        })
        .collect();

    Ok(converted)
}

/// Traduit un résultat de sonde en résultat présentable à l'analyste.
///
/// C'est ici que se joue la seule règle qui compte : **une sonde dégradée ne
/// conclut rien**. Si sa règle répond « existe » à un pseudo aléatoire, son
/// « existe » sur la vraie cible ne vaut pas davantage — et son « inexistant »
/// non plus, puisque la règle est cassée dans les deux sens. Les deux verdicts
/// conclusifs sont donc ramenés à `Indeterminate`, motif à l'appui.
///
/// Les états `Blocked`, `Indeterminate` et `Error` sont laissés intacts : ils
/// ne concluaient déjà rien, les réécrire n'ajouterait que du bruit.
fn present_result(
    result: spectra_probe::ProbeResult,
    degradation: Option<String>,
) -> ProbeResult {
    let outcome = match result.outcome {
        SpectraOutcome::Exists { .. } | SpectraOutcome::Missing
            if degradation.is_some() =>
        {
            ProbeOutcome::Indeterminate {
                reason: format!(
                    "sonde dégradée — {}",
                    degradation.as_deref().unwrap_or("règle non fiable")
                ),
            }
        }
        // `final_url` est l'URL après redirections : c'est elle qui pointe vers
        // le profil réel, pas l'URL de la sonde.
        SpectraOutcome::Exists { evidence, .. } => ProbeOutcome::Exists {
            url: Some(evidence.final_url),
        },
        SpectraOutcome::Missing => ProbeOutcome::Missing,
        SpectraOutcome::Blocked { reason } => ProbeOutcome::Blocked {
            reason: format!("{reason:?}"),
        },
        SpectraOutcome::Indeterminate { reason } => ProbeOutcome::Indeterminate { reason },
        SpectraOutcome::Error(e) => ProbeOutcome::Error {
            message: format!("{e:?}"),
        },
    };

    ProbeResult {
        site: result
            .probe_id
            .0
            .split(':')
            .nth(1)
            .unwrap_or("unknown")
            .to_string(),
        outcome,
        elapsed_ms: result.elapsed_ms,
        degraded: degradation.is_some(),
    }
}

/// Liste les sondes disponibles, pour que l'analyste sache ce qui sera interrogé.
#[command]
pub async fn load_osint_probes(
    state: tauri::State<'_, AppState>,
) -> AppResult<Vec<ProbeInfo>> {
    let probes = load_probes(state.storage_root())?;

    Ok(probes
        .into_iter()
        .map(|p| ProbeInfo {
            id: p.id.0,
            site_name: p.site_name,
            category: format!("{:?}", p.category).to_lowercase(),
            selector_kind: format!("{:?}", p.selector_kind).to_lowercase(),
            license: p.source.license,
            source: p.source.source,
        })
        .collect())
}

/// Description d'une sonde exposée au frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProbeInfo {
    pub id: String,
    pub site_name: String,
    pub category: String,
    pub selector_kind: String,
    /// Licence du descripteur amont — doit rester visible (CLAUDE.md §8).
    pub license: String,
    pub source: String,
}

/// Met à jour les datasets OSINT.
#[command]
pub async fn update_osint_datasets(
    _state: tauri::State<'_, AppState>,
) -> AppResult<UpdateReport> {
    // À implémenter : télécharger les datasets amont
    Ok(UpdateReport {
        added: 0,
        removed: 0,
        modified: 0,
        errors: vec![],
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateReport {
    pub added: usize,
    pub removed: usize,
    pub modified: usize,
    pub errors: Vec<String>,
}

// =============================================================================
// Bilan de santé des sondes (anti-faux-positifs)
// =============================================================================

/// Durée de validité d'un bilan de santé.
///
/// La santé d'une sonde est une propriété de la *règle*, pas de la recherche :
/// la recontrôler à chaque requête de l'analyste doublerait le trafic sortant
/// sans rien apprendre de neuf. Une demi-heure suffit à détecter un site qui
/// change de comportement, sans transformer chaque recherche en double campagne.
const HEALTH_TTL: std::time::Duration = std::time::Duration::from_secs(30 * 60);

/// Sondes reconnues dégradées, indexées par identifiant → raison.
static HEALTH: std::sync::OnceLock<std::sync::Mutex<HealthCache>> =
    std::sync::OnceLock::new();

#[derive(Default)]
struct HealthCache {
    checked_at: Option<std::time::Instant>,
    degraded: std::collections::HashMap<String, String>,
}

/// Identifie les sondes dont la règle de décision est cassée.
///
/// Le principe (CLAUDE.md §8) : on interroge chaque sonde avec un sélecteur
/// aléatoire qui n'existe statistiquement nulle part. Une sonde saine répond
/// « inexistant ». Une sonde qui répond « existe » signalerait un compte sur
/// *toutes* les recherches — le défaut le plus grave possible dans un outil
/// d'enquête, et celui qu'aucune relecture de code ne révèle : il vient du
/// site, pas du nôtre. C'est exactement ainsi qu'a été détecté le cas PyPI,
/// qui sert sa page anti-bot avec un code HTTP 200.
async fn degraded_probes(
    engine: &ProbeEngine,
    probes: &[spectra_probe::Probe],
) -> std::collections::HashMap<String, String> {
    let cache = HEALTH.get_or_init(Default::default);

    // Le verrou est relâché avant tout `await` : le tenir à travers une
    // campagne réseau bloquerait toutes les autres recherches pendant l'I/O.
    {
        let guard = cache.lock().unwrap_or_else(|e| e.into_inner());
        if guard
            .checked_at
            .is_some_and(|t| t.elapsed() < HEALTH_TTL)
        {
            return guard.degraded.clone();
        }
    }

    // Toutes les sondes filtrées partagent le même type de sélecteur : un seul
    // sélecteur de contrôle suffit pour la campagne entière.
    let Some(kind) = probes.first().map(|p| p.selector_kind.clone()) else {
        return std::collections::HashMap::new();
    };
    let control = spectra_probe::control::generate_control_selector(&kind);

    let cancel = tokio_util::sync::CancellationToken::new();
    let results = engine.run_campaign(&control, probes, cancel).await;

    let degraded: std::collections::HashMap<String, String> = results
        .into_iter()
        .filter_map(|r| match r.outcome {
            SpectraOutcome::Exists { .. } => Some((
                r.probe_id.0,
                "répond « existe » sur un pseudo aléatoire".to_string(),
            )),
            _ => None,
        })
        .collect();

    {
        let mut guard = cache.lock().unwrap_or_else(|e| e.into_inner());
        guard.checked_at = Some(std::time::Instant::now());
        guard.degraded = degraded.clone();
    }

    degraded
}

// =============================================================================
// Chargement des sondes
// =============================================================================

/// Snapshot embarqué dans le binaire.
///
/// CLAUDE.md §9 : l'application doit fonctionner **à l'installation, hors
/// ligne, sans rien télécharger**. Les sondes sont donc compilées dans
/// l'exécutable plutôt que lues depuis un chemin qui pourrait ne pas exister.
const EMBEDDED_PROBES: &str = include_str!("../../../data/probes/probes.json");

/// Charge les sondes : surcharge locale si présente, sinon snapshot embarqué.
///
/// La surcharge vit dans le répertoire de données applicatives, à côté du
/// magasin de preuves — c'est là qu'écrira la mise à jour manuelle des
/// datasets. Le snapshot embarqué garantit qu'une installation neuve
/// fonctionne immédiatement.
fn load_probes(storage_root: &std::path::Path) -> AppResult<Vec<spectra_probe::Probe>> {
    let override_path = storage_root
        .parent()
        .map(|p| p.join("probes").join("probes.json"));

    if let Some(path) = override_path {
        if path.is_file() {
            let content = std::fs::read_to_string(&path).map_err(|e| {
                AppError::msg(format!("lecture des sondes ({}) : {e}", path.display()))
            })?;
            return serde_json::from_str(&content).map_err(|e| {
                AppError::msg(format!("sondes illisibles ({}) : {e}", path.display()))
            });
        }
    }

    serde_json::from_str(EMBEDDED_PROBES)
        .map_err(|e| AppError::msg(format!("snapshot de sondes embarqué invalide : {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use spectra_probe::schema::{Evidence, ProbeId};

    fn result_with(outcome: SpectraOutcome) -> spectra_probe::ProbeResult {
        spectra_probe::ProbeResult {
            probe_id: ProbeId("builtin:exemple".to_string()),
            selector: "torvalds".to_string(),
            outcome,
            elapsed_ms: 42,
        }
    }

    fn exists() -> SpectraOutcome {
        SpectraOutcome::Exists {
            evidence: Evidence {
                triggered_assertion: Some("status_code=200".to_string()),
                status_code: 200,
                final_url: "https://exemple.test/torvalds".to_string(),
                body_hash: "abcd".to_string(),
                body_preview: None,
                timestamp: chrono::Utc::now(),
            },
            extracted: Default::default(),
        }
    }

    #[test]
    fn sonde_saine_conserve_son_verdict() {
        let r = present_result(result_with(exists()), None);
        assert!(matches!(r.outcome, ProbeOutcome::Exists { .. }));
        assert!(!r.degraded);
        assert_eq!(r.site, "exemple");
    }

    /// Le cas qui justifie tout le mécanisme : sans cette conversion, une règle
    /// cassée verse un compte inexistant au dossier d'enquête.
    #[test]
    fn sonde_degradee_ne_peut_pas_affirmer_une_existence() {
        let r = present_result(result_with(exists()), Some("règle cassée".to_string()));

        assert!(
            !matches!(r.outcome, ProbeOutcome::Exists { .. }),
            "une sonde dégradée ne doit jamais présenter un compte comme trouvé"
        );
        match r.outcome {
            ProbeOutcome::Indeterminate { reason } => {
                assert!(reason.contains("dégradée"), "motif illisible : {reason}");
            }
            other => panic!("attendu Indeterminate, obtenu {other:?}"),
        }
        assert!(r.degraded);
    }

    /// Une règle cassée l'est dans les deux sens : son « inexistant » ne prouve
    /// pas davantage une absence de compte.
    #[test]
    fn sonde_degradee_ne_peut_pas_affirmer_une_absence() {
        let r = present_result(
            result_with(SpectraOutcome::Missing),
            Some("règle cassée".to_string()),
        );
        assert!(matches!(r.outcome, ProbeOutcome::Indeterminate { .. }));
    }

    /// `Blocked` ne concluait déjà rien : le réécrire ferait perdre la raison
    /// du blocage, qui est l'information utile à l'analyste.
    #[test]
    fn etat_deja_non_conclusif_reste_intact() {
        let r = present_result(
            result_with(SpectraOutcome::Blocked {
                reason: spectra_probe::schema::BlockReason::Cloudflare,
            }),
            Some("règle cassée".to_string()),
        );
        match r.outcome {
            ProbeOutcome::Blocked { reason } => assert!(reason.contains("Cloudflare")),
            other => panic!("attendu Blocked, obtenu {other:?}"),
        }
        // Le drapeau reste levé : l'UI doit toujours signaler la sonde.
        assert!(r.degraded);
    }
}
