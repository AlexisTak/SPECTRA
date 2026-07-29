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

    let cancel = tokio_util::sync::CancellationToken::new();
    let results = engine.run_campaign(&selector, &filtered, cancel).await;

    // Convertir les résultats
    let converted: Vec<ProbeResult> = results
        .into_iter()
        .map(|r| {
            let outcome = match r.outcome {
                // `final_url` est l'URL après redirections : c'est elle qui
                // pointe vers le profil réel, pas l'URL de la sonde.
                SpectraOutcome::Exists { evidence, .. } => ProbeOutcome::Exists {
                    url: Some(evidence.final_url),
                },
                SpectraOutcome::Missing => ProbeOutcome::Missing,
                SpectraOutcome::Blocked { reason } => ProbeOutcome::Blocked {
                    reason: format!("{reason:?}"),
                },
                SpectraOutcome::Indeterminate { reason } => {
                    ProbeOutcome::Indeterminate { reason }
                }
                SpectraOutcome::Error(e) => ProbeOutcome::Error {
                    message: format!("{e:?}"),
                },
            };

            ProbeResult {
                site: r.probe_id.0.split(':').nth(1).unwrap_or("unknown").to_string(),
                outcome,
                elapsed_ms: r.elapsed_ms,
            }
        })
        .collect();

    Ok(converted)
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
