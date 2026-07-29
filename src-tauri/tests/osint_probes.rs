//! Vérifie que le snapshot de sondes embarqué est utilisable.
//!
//! Le fichier est inclus dans le binaire par `include_str!` : une erreur de
//! syntaxe ou un champ manquant ne se manifesterait qu'à l'exécution, au
//! moment où l'analyste lance une recherche. Ces tests la font apparaître au
//! build.

use spectra_probe::{Assertion, Probe, SelectorKind};

/// Même chemin que celui utilisé par `osint.rs`.
const EMBEDDED_PROBES: &str = include_str!("../../data/probes/probes.json");

fn load() -> Vec<Probe> {
    serde_json::from_str(EMBEDDED_PROBES).expect("le snapshot embarqué doit être désérialisable")
}

#[test]
fn le_snapshot_embarque_est_deserialisable() {
    let probes = load();
    assert!(
        !probes.is_empty(),
        "un snapshot vide rendrait la recherche inopérante sans le signaler"
    );
}

#[test]
fn chaque_sonde_substitue_le_selecteur() {
    for probe in load() {
        assert!(
            probe.request.url.contains("{selector}"),
            "{} : l'URL doit contenir {{selector}}, sinon toutes les recherches \
             interrogeraient la même page",
            probe.id.0
        );
    }
}

#[test]
fn chaque_sonde_sait_decider_dans_les_deux_sens() {
    for probe in load() {
        assert!(
            !probe.decision.exists_if.is_empty(),
            "{} : sans règle d'existence, la sonde ne peut jamais conclure",
            probe.id.0
        );
        assert!(
            !probe.decision.missing_if.is_empty(),
            "{} : sans règle d'absence, tout résultat négatif devient \
             Indeterminate — la sonde est inutile",
            probe.id.0
        );
    }
}

#[test]
fn les_identifiants_sont_uniques() {
    let probes = load();
    let mut ids: Vec<&str> = probes.iter().map(|p| p.id.0.as_str()).collect();
    ids.sort_unstable();
    let before = ids.len();
    ids.dedup();

    assert_eq!(
        before,
        ids.len(),
        "des identifiants dupliqués rendraient le cache et le score de qualité \
         incohérents"
    );
}

#[test]
fn les_regles_regex_compilent() {
    for probe in load() {
        let assertions = probe
            .decision
            .exists_if
            .iter()
            .chain(probe.decision.missing_if.iter())
            .chain(probe.decision.blocked_if.iter());

        for assertion in assertions {
            if let Assertion::BodyMatches(pattern) = assertion {
                assert!(
                    regex::Regex::new(pattern).is_ok(),
                    "{} : regex invalide « {pattern} » — l'assertion échouerait \
                     silencieusement à l'exécution",
                    probe.id.0
                );
            }
        }
    }
}

#[test]
fn la_licence_amont_est_tracee() {
    // CLAUDE.md §8 : chaque sonde conserve sa provenance et sa licence, sans
    // quoi l'attribution devient impossible à produire.
    for probe in load() {
        assert!(
            !probe.source.license.trim().is_empty(),
            "{} : licence manquante",
            probe.id.0
        );
        assert!(
            !probe.source.source.trim().is_empty(),
            "{} : source amont manquante",
            probe.id.0
        );
    }
}

#[test]
fn le_snapshot_couvre_la_recherche_par_pseudo() {
    let probes = load();
    let usernames = probes
        .iter()
        .filter(|p| matches!(p.selector_kind, SelectorKind::Username))
        .count();

    assert!(
        usernames > 0,
        "sans sonde de type Username, l'écran OSINT ne renverrait jamais rien"
    );
}
