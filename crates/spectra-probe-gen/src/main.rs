//! Générateur de sondes OSINT.
//!
//! Convertit les datasets amont (WhatsMyName, Sherlock, Maigret) vers le
//! schéma interne spectra-probe. Exécuté au build, pas au runtime.
//!
//! Usage: cargo run -p spectra-probe-gen -- --whatsmyname data/whatsmyname/wmn-data.json

mod wmn;
mod report;

use clap::Parser;
use report::ConversionReport;
use spectra_probe::Probe;
use std::path::PathBuf;
use std::fs;
use sha2::{Sha256, Digest};

#[derive(Parser, Debug)]
#[command(name = "spectra-probe-gen")]
#[command(about = "Générateur de sondes OSINT")]
struct Args {
    /// Fichier WhatsMyName wmn-data.json
    #[arg(long)]
    whatsmyname: Option<PathBuf>,

    /// Fichier Sherlock data.json
    #[arg(long)]
    sherlock: Option<PathBuf>,

    /// Fichier Maigret data.json
    #[arg(long)]
    maigret: Option<PathBuf>,

    /// Dossier de sortie des sondes générées
    #[arg(long, default_value = "data/probes")]
    output: PathBuf,
}

fn main() {
    tracing_subscriber::fmt::init();

    let args = Args::parse();
    let mut report = ConversionReport::new();
    let mut all_probes: Vec<Probe> = Vec::new();

    // WhatsMyName
    if let Some(path) = &args.whatsmyname {
        println!("Conversion WhatsMyName: {}", path.display());
        match fs::read_to_string(path) {
            Ok(content) => {
                let hash = hash_file(&content);
                match wmn::convert(&content, &hash) {
                    Ok((probes, sub_report)) => {
                        let rejected: usize = sub_report.sources.values().map(|s| s.rejected).sum();
                        println!("  {} sondes converties, {} rejetées", probes.len(), rejected);
                        all_probes.extend(probes);
                        report.merge(sub_report);
                    }
                    Err(e) => {
                        eprintln!("  Erreur: {}", e);
                        report.error("whatsmyname", e.to_string());
                    }
                }
            }
            Err(e) => {
                eprintln!("  Lecture impossible: {}", e);
                report.error("whatsmyname", e.to_string());
            }
        }
    }


    // Écrire le rapport
    fs::create_dir_all(&args.output).expect("créer dossier output");
    let report_path = args.output.join("conversion-report.json");
    fs::write(&report_path, serde_json::to_string_pretty(&report).unwrap())
        .expect("écrire rapport");
    println!("\nRapport: {}", report_path.display());

    // Sérialiser les sondes
    let probes_path = args.output.join("probes.json");
    let probes_json = serde_json::to_string_pretty(&all_probes).unwrap();
    fs::write(&probes_path, &probes_json).expect("écrire sondes");
    println!("Sondes: {} ({} octets)", probes_path.display(), probes_json.len());

    // Afficher le résumé
    println!("\n{}", report.summary());
}

fn hash_file(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    hex::encode(hasher.finalize())
}
