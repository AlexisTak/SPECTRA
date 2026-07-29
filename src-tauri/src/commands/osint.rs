//! Commandes OSINT — pont entre Tauri et spectra-probe.

use crate::database::AppState;
use crate::error::AppResult;
use serde::{Deserialize, Serialize};
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsintProgress {
    pub current: usize,
    pub total: usize,
    pub result: ProbeResult,
}

/// Lance une campagne OSINT sur un sélecteur donné.
///
/// Retourne un flux de résultats via un canal mpsc.
#[command]
pub async fn run_osint_campaign(
    state: tauri::State<'_, AppState>,
    selector: String,
    kind: SelectorKind,
) -> AppResult<String> {
    // Pour l'instant, retourne des résultats mockés
    // Le branchement à spectra-probe se fera via:
    // 1. Chargement des sondes depuis data/probes/probes.json
    // 2. Exécution via ProbeEngine::run_campaign
    // 3. Stream des résultats via tokio::sync::mpsc

    let _conn = state.get_conn().await;
    let _kind = kind;
    let _selector = selector;

    // Mock results — à remplacer par le vrai moteur
    Ok(serde_json::to_string(&mock_campaign())?)
}

fn mock_campaign() -> Vec<ProbeResult> {
    use rand::Rng;
    let mut rng = rand::rng();

    let sites = vec![
        "GitHub", "Twitter", "Instagram", "Reddit", "TikTok",
        "Facebook", "LinkedIn", "Pinterest", "Snapchat", "YouTube",
        "Tumblr", "Vimeo", "Flickr", "SoundCloud", "Spotify",
    ];

    sites
        .into_iter()
        .map(|site| {
            let r = rng.random::<f32>();
            let outcome = if r > 0.7 {
                ProbeOutcome::Exists {
                    url: Some(format!("https://{}.com/{}", site.to_lowercase(), "user")),
                }
            } else if r > 0.6 {
                ProbeOutcome::Blocked {
                    reason: "Cloudflare detected".to_string(),
                }
            } else if r > 0.5 {
                ProbeOutcome::Indeterminate {
                    reason: "Response ambiguous".to_string(),
                }
            } else {
                ProbeOutcome::Missing
            };

            ProbeResult {
                site: site.to_string(),
                outcome,
                elapsed_ms: rng.random_range(100..2000),
            }
        })
        .collect()
}

/// Charge les sondes depuis le fichier généré par spectra-probe-gen.
#[command]
pub async fn load_osint_probes() -> AppResult<Vec<serde_json::Value>> {
    // À implémenter : lire data/probes/probes.json
    Ok(vec![])
}

/// Met à jour les datasets OSINT (WhatsMyName, Sherlock, Maigret).
#[command]
pub async fn update_osint_datasets(
    _state: tauri::State<'_, AppState>,
) -> AppResult<UpdateReport> {
    // À implémenter : télécharger les datasets amont,
    // lancer spectra-probe-gen, retourner un rapport
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
