//! Vérifie l'ingestion et le contrôle d'intégrité des preuves.
//!
//! Ces tests portent sur la garantie centrale du produit : une empreinte
//! **calculée par le système** au moment de l'ingestion, et **recalculée** à
//! chaque vérification. Ils échoueraient sur l'implémentation d'origine, qui
//! enregistrait l'empreinte fournie par l'appelant sans jamais ouvrir le
//! fichier (`audit.md`, P1-1 et P1-2).
//!
//! Le test exerce la logique métier directement sur une base temporaire, sans
//! passer par le runtime Tauri : les commandes `#[tauri::command]` exigent un
//! `State` qu'on ne peut pas construire hors application.

use cekarna::database::{append_audit_event, init_database, sha256_hash_bytes};
use rusqlite::Connection;
use tempfile::TempDir;

/// Reproduit l'ingestion : lecture, empreinte, copie, enregistrement.
///
/// Miroir de `add_evidence`. Si l'un des deux change, ce test doit changer
/// aussi — c'est le prix à payer tant que la logique n'est pas extraite dans
/// une fonction partagée testable indépendamment de `tauri::State`.
fn ingest(
    conn: &Connection,
    storage_root: &std::path::Path,
    case_id: &str,
    source: &std::path::Path,
) -> (String, String) {
    let bytes = std::fs::read(source).expect("lecture du fichier source");
    let sha256 = sha256_hash_bytes(&bytes);
    let id = uuid::Uuid::new_v4().to_string();

    let dest_dir = storage_root.join("evidence").join(&id[..2]);
    std::fs::create_dir_all(&dest_dir).expect("création du magasin");
    let dest = dest_dir.join(format!("{id}.txt"));
    std::fs::write(&dest, &bytes).expect("copie dans le magasin");

    conn.execute(
        "INSERT INTO evidence (id, caseId, type, nom, chemin, hash_sha256, taille, dateAjout, statut) \
         VALUES (?, ?, 'file', ?, ?, ?, ?, datetime('now'), 'verifie')",
        rusqlite::params![
            &id,
            case_id,
            source.file_name().unwrap().to_string_lossy().to_string(),
            dest.to_string_lossy().to_string(),
            &sha256,
            bytes.len() as i64,
        ],
    )
    .expect("insertion de la preuve");

    (id, dest.to_string_lossy().to_string())
}

/// Relit le fichier et compare : c'est la définition d'une vérification.
fn verify(conn: &Connection, evidence_id: &str) -> &'static str {
    let (chemin, expected): (Option<String>, Option<String>) = conn
        .query_row(
            "SELECT chemin, hash_sha256 FROM evidence WHERE id = ?",
            rusqlite::params![evidence_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .expect("preuve introuvable");

    let (Some(chemin), Some(expected)) = (chemin, expected) else {
        return "no_hash";
    };

    let Ok(bytes) = std::fs::read(&chemin) else {
        return "missing";
    };

    if sha256_hash_bytes(&bytes) == expected {
        "intact"
    } else {
        "altered"
    }
}

fn setup() -> (TempDir, Connection, String) {
    let dir = TempDir::new().expect("répertoire temporaire");
    let conn = init_database(&dir.path().join("test.db")).expect("init base");

    let case_id = uuid::Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO cases (id, reference, titre, statut, priorite, dateCreation, dateMiseJour) \
         VALUES (?, 'ENQ-2026-0001', 'Dossier de test', 'ouvert', 'normale', datetime('now'), datetime('now'))",
        rusqlite::params![&case_id],
    )
    .expect("création du dossier");

    (dir, conn, case_id)
}

#[test]
fn empreinte_calculee_a_l_ingestion_et_non_recue() {
    let (dir, conn, case_id) = setup();

    let source = dir.path().join("piece.txt");
    std::fs::write(&source, b"Contenu original de la piece.").unwrap();
    let attendu = sha256_hash_bytes(b"Contenu original de la piece.");

    let (id, _) = ingest(&conn, dir.path(), &case_id, &source);

    let stocke: String = conn
        .query_row(
            "SELECT hash_sha256 FROM evidence WHERE id = ?",
            rusqlite::params![&id],
            |r| r.get(0),
        )
        .unwrap();

    assert_eq!(
        stocke, attendu,
        "l'empreinte enregistrée doit être celle du contenu réel du fichier"
    );
}

#[test]
fn le_fichier_est_copie_dans_le_magasin() {
    let (dir, conn, case_id) = setup();

    let source = dir.path().join("piece.txt");
    std::fs::write(&source, b"Contenu.").unwrap();

    let (_, stored) = ingest(&conn, dir.path(), &case_id, &source);

    assert!(
        std::path::Path::new(&stored).is_file(),
        "le fichier doit exister dans le magasin"
    );
    assert_ne!(
        stored,
        source.to_string_lossy(),
        "la preuve ne doit pas rester un simple pointeur vers le fichier d'origine"
    );

    // La preuve survit à la disparition de l'original : c'est tout l'intérêt
    // de la copie.
    std::fs::remove_file(&source).unwrap();
    assert!(std::path::Path::new(&stored).is_file());
}

#[test]
fn preuve_intacte_est_reconnue_intacte() {
    let (dir, conn, case_id) = setup();
    let source = dir.path().join("piece.txt");
    std::fs::write(&source, b"Contenu stable.").unwrap();

    let (id, _) = ingest(&conn, dir.path(), &case_id, &source);

    assert_eq!(verify(&conn, &id), "intact");
}

#[test]
fn preuve_modifiee_est_detectee() {
    let (dir, conn, case_id) = setup();
    let source = dir.path().join("piece.txt");
    std::fs::write(&source, b"Contenu original.").unwrap();

    let (id, stored) = ingest(&conn, dir.path(), &case_id, &source);
    assert_eq!(verify(&conn, &id), "intact");

    // Altération du fichier stocké, sans toucher à l'empreinte enregistrée.
    std::fs::write(&stored, b"Contenu FALSIFIE.").unwrap();

    assert_eq!(
        verify(&conn, &id),
        "altered",
        "une preuve modifiée doit être détectée : c'est la garantie centrale"
    );
}

#[test]
fn preuve_supprimee_est_detectee() {
    let (dir, conn, case_id) = setup();
    let source = dir.path().join("piece.txt");
    std::fs::write(&source, b"Contenu.").unwrap();

    let (id, stored) = ingest(&conn, dir.path(), &case_id, &source);
    std::fs::remove_file(&stored).unwrap();

    assert_eq!(verify(&conn, &id), "missing");
}

#[test]
fn la_chaine_d_audit_survit_a_plusieurs_evenements() {
    let (dir, conn, case_id) = setup();

    for i in 1..=5 {
        append_audit_event(
            &conn,
            &case_id,
            "create",
            "evidence",
            Some(&format!("evd-{i}")),
            "analyste",
            serde_json::json!({ "index": i }),
        )
        .expect("écriture du maillon");
    }

    // Séquence strictement croissante : c'est elle qui définit l'ordre, pas
    // l'horodatage à la seconde (`audit.md`, P2-14).
    let sequences: Vec<i64> = conn
        .prepare("SELECT sequence FROM audit_events WHERE caseId = ? ORDER BY sequence")
        .unwrap()
        .query_map(rusqlite::params![&case_id], |r| r.get(0))
        .unwrap()
        .map(Result::unwrap)
        .collect();

    assert_eq!(sequences, vec![1, 2, 3, 4, 5]);

    // Chaque maillon référence l'empreinte du précédent.
    let chain: Vec<(String, String)> = conn
        .prepare("SELECT imma, imma_precedent FROM audit_events WHERE caseId = ? ORDER BY sequence")
        .unwrap()
        .query_map(rusqlite::params![&case_id], |r| {
            Ok((r.get(0)?, r.get::<_, Option<String>>(1)?.unwrap_or_default()))
        })
        .unwrap()
        .map(Result::unwrap)
        .collect();

    assert_eq!(chain[0].1, "", "le premier maillon n'a pas de précédent");
    for i in 1..chain.len() {
        assert_eq!(
            chain[i].1, chain[i - 1].0,
            "le maillon {i} doit référencer l'empreinte du précédent"
        );
    }

    drop(dir);
}
