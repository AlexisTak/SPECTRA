//! Vérification manuelle des sondes contre les sites réels.
//!
//! **Cet exemple sort sur le réseau.** Il n'est donc pas un test : la suite de
//! tests ne doit jamais dépendre de la disponibilité d'un site tiers
//! (CLAUDE.md §11). Il sert à constater, ponctuellement, si les règles de
//! décision embarquées correspondent encore au comportement des sites.
//!
//! Usage :
//! ```text
//! cargo run -p spectra-probe --example live_check -- <pseudo>
//! ```
//!
//! La sonde de contrôle est exécutée d'abord : un pseudo aléatoire qui ne doit
//! exister nulle part. Si une sonde répond « existe » dessus, sa règle est
//! cassée et son verdict sur la vraie cible ne vaut rien.

use spectra_probe::{Probe, ProbeEngine, ProbeOutcome};

const EMBEDDED_PROBES: &str = include_str!("../../../data/probes/probes.json");

#[tokio::main]
async fn main() {
    let target = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("usage: cargo run -p spectra-probe --example live_check -- <pseudo>");
        std::process::exit(2);
    });

    let probes: Vec<Probe> =
        serde_json::from_str(EMBEDDED_PROBES).expect("snapshot de sondes illisible");

    let engine = ProbeEngine::new(
        "Cekarna/0.1.0 (outil d'investigation OSINT; +https://github.com/cekarna/enquetes)"
            .to_string(),
        8,
    );

    // 1. Sonde de contrôle — la règle est-elle saine ?
    let control = spectra_probe::control::generate_control_selector(
        &spectra_probe::SelectorKind::Username,
    );
    println!("Contrôle avec « {control} » (ne doit exister nulle part)\n");

    let cancel = tokio_util::sync::CancellationToken::new();
    let control_results = engine.run_campaign(&control, &probes, cancel.clone()).await;

    let mut degraded = Vec::new();
    for r in &control_results {
        let verdict = match &r.outcome {
            ProbeOutcome::Missing => "sain",
            ProbeOutcome::Exists { .. } => {
                degraded.push(r.probe_id.0.clone());
                "DÉGRADÉE — répond « existe » sur un pseudo aléatoire"
            }
            ProbeOutcome::Blocked { reason } => {
                println!("  {:<24} bloquée ({reason:?})", r.probe_id.0);
                continue;
            }
            ProbeOutcome::Indeterminate { .. } => "indéterminée",
            ProbeOutcome::Error(e) => {
                println!("  {:<24} erreur ({e:?})", r.probe_id.0);
                continue;
            }
        };
        println!("  {:<24} {verdict}", r.probe_id.0);
    }

    // 2. Cible réelle
    println!("\nRecherche de « {target} »\n");
    let results = engine.run_campaign(&target, &probes, cancel).await;

    let mut found = 0;
    for r in &results {
        // Une sonde dégradée ne peut pas conclure, quel que soit son verdict.
        if degraded.contains(&r.probe_id.0) {
            println!("  {:<24} ignorée (sonde dégradée)", r.probe_id.0);
            continue;
        }

        match &r.outcome {
            ProbeOutcome::Exists { evidence, .. } => {
                found += 1;
                println!("  {:<24} TROUVÉ  {}", r.probe_id.0, evidence.final_url);
            }
            ProbeOutcome::Missing => println!("  {:<24} absent", r.probe_id.0),
            ProbeOutcome::Blocked { reason } => {
                println!("  {:<24} bloqué ({reason:?})", r.probe_id.0)
            }
            ProbeOutcome::Indeterminate { reason } => {
                println!("  {:<24} indéterminé ({reason})", r.probe_id.0)
            }
            ProbeOutcome::Error(e) => println!("  {:<24} erreur ({e:?})", r.probe_id.0),
        }
    }

    println!(
        "\n{found} compte(s) trouvé(s) sur {} sonde(s), {} dégradée(s)",
        results.len(),
        degraded.len()
    );
}
