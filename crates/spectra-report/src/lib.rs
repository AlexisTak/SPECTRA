//! Génération de rapports SPECTRA.
//!
//! Produits supportés :
//! - Markdown
//! - HTML (avec style embarqué)
//! - PDF (via `printpdf`, polices intégrées, pas de dépendance externe)
//!
//! Chaque rapport inclut :
//! - métadonnées du dossier,
//! - table des entités découvertes,
//! - observations groupées par entité,
//! - relations,
//! - journal d'audit (si disponible),
//! - annexes de preuves avec hashes.

use serde::{Deserialize, Serialize};
use spectra_core::{Entity, Observation, Provenance};
use spectra_store::Store;
use std::collections::BTreeMap;
use std::io::BufWriter;

/// Erreur de génération de rapport.
#[derive(Debug, thiserror::Error)]
pub enum ReportError {
    /// Erreur du store.
    #[error("erreur de stockage : {0}")]
    Store(#[from] spectra_store::StoreError),
    /// Erreur JSON.
    #[error("erreur JSON : {0}")]
    Json(#[from] serde_json::Error),
    /// Erreur PDF.
    #[error("erreur PDF : {0}")]
    Pdf(String),
}

impl From<printpdf::Error> for ReportError {
    fn from(err: printpdf::Error) -> Self {
        ReportError::Pdf(err.to_string())
    }
}

/// Options de génération d'un rapport.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportOptions {
    /// Titre du rapport.
    pub title: String,
    /// Identifiant du dossier.
    pub case_id: String,
    /// Nom de l'opérateur.
    pub operator: String,
    /// Inclure les observations inférées.
    pub include_inferred: bool,
    /// Format de sortie.
    pub format: ReportFormat,
}

/// Format de sortie du rapport.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReportFormat {
    /// Markdown brut, réimportable et diffable.
    Markdown,
    /// HTML autonome, consultable hors ligne.
    Html,
    /// PDF paginé, destiné à la remise.
    Pdf,
    /// DOCX, pour relecture et annotation.
    Docx,
}

/// Moteur de rapports.
pub struct ReportEngine;

impl ReportEngine {
    /// Crée un nouveau moteur.
    pub fn new() -> Self {
        Self
    }

    /// Génère un rapport textuel (Markdown ou HTML) depuis un store.
    pub fn generate(
        &self,
        store: &Store,
        opts: &ReportOptions,
    ) -> Result<String, ReportError> {
        let entities = store.list_entities()?;
        let mut entity_obs: BTreeMap<String, Vec<Observation>> = BTreeMap::new();
        for e in &entities {
            let obs = store.get_observations_by_subject(&e.id)?;
            entity_obs.insert(e.id.clone(), obs);
        }

        match opts.format {
            ReportFormat::Markdown => {
                Ok(self.to_markdown(opts, &entities, &entity_obs))
            }
            ReportFormat::Html => {
                Ok(self.to_html(opts, &entities, &entity_obs))
            }
            ReportFormat::Pdf => Err(ReportError::Pdf(
                "utilisez generate_pdf() pour le format PDF".to_string(),
            )),
            ReportFormat::Docx => Err(ReportError::Pdf(
                "utilisez generate_docx() pour le format DOCX".to_string(),
            )),
        }
    }

    /// Génère un rapport PDF complet sous forme de bytes.
    pub fn generate_pdf(
        &self,
        store: &Store,
        opts: &ReportOptions,
    ) -> Result<Vec<u8>, ReportError> {
        let entities = store.list_entities()?;
        let mut entity_obs: BTreeMap<String, Vec<Observation>> = BTreeMap::new();
        for e in &entities {
            let obs = store.get_observations_by_subject(&e.id)?;
            entity_obs.insert(e.id.clone(), obs);
        }

        let mut pdf = PdfReport::new()?;

        // Page de garde
        pdf.draw_title(&opts.title, 24.0);
        pdf.draw_line(&format!("Dossier : {}", opts.case_id), 12.0);
        pdf.draw_line(&format!("Opérateur : {}", opts.operator), 12.0);
        pdf.draw_line(
            &format!(
                "Date : {}",
                chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
            ),
            12.0,
        );
        pdf.new_page();

        // Résumé
        pdf.draw_title("Résumé", 16.0);
        let total_obs = entity_obs.values().map(|v| v.len()).sum::<usize>();
        pdf.draw_line(&format!("{} entités, {} observations.", entities.len(), total_obs), 11.0);
        pdf.new_page();

        // Entités et observations
        pdf.draw_title("Entités et observations", 16.0);
        for e in &entities {
            pdf.draw_line(&format!("{} ({:?}) — {}", e.display_label, e.kind, e.canonical_value), 11.0);
            if let Some(obs) = entity_obs.get(&e.id) {
                for o in obs {
                    if !opts.include_inferred && o.provenance == Provenance::Inferred {
                        continue;
                    }
                    let provenance_label = format!("{:?}", o.provenance);
                    let val = serde_json::to_string(&o.value)
                        .unwrap_or_default()
                        .trim_matches('"')
                        .to_string();
                    let line = format!(
                        "  • {} = {} (source: {}, confiance: {:?}, provenance: {})",
                        o.predicate, val, o.source, o.confidence, provenance_label
                    );
                    let color = match o.provenance {
                        Provenance::Inferred => printpdf::Color::Rgb(printpdf::Rgb::new(
                            0.4, 0.4, 0.4, None,
                        )),
                        _ => printpdf::Color::Rgb(printpdf::Rgb::new(
                            0.0, 0.5, 0.0, None,
                        )),
                    };
                    pdf.draw_colored_line(&line, 10.0, color);
                }
            }
            pdf.advance(4.0);
        }
        pdf.new_page();

        // Journal d'audit
        pdf.draw_title("Journal d'audit", 16.0);
        let audit_events = store.list_audit_events(&opts.case_id).unwrap_or_default();
        if audit_events.is_empty() {
            pdf.draw_line("Aucun événement d'audit enregistré.", 11.0);
        } else {
            for ev in &audit_events {
                pdf.draw_line(
                    &format!(
                        "[{}] {} — {} / {} (séq: {}, hash: {}…{})",
                        ev.timestamp,
                        ev.action,
                        ev.entity_kind,
                        ev.actor,
                        ev.sequence,
                        &ev.hash[..8.min(ev.hash.len())],
                        &ev.hash[ev.hash.len().saturating_sub(8)..]
                    ),
                    9.0,
                );
            }
        }
        pdf.new_page();

        // Annexes : preuves avec hashes bruts
        pdf.draw_title("Annexes — Empreintes de preuves", 16.0);
        let mut hashes: Vec<String> = Vec::new();
        for obs_list in entity_obs.values() {
            for o in obs_list {
                if let Some(rh) = &o.raw_hash {
                    hashes.push(format!("{} : {}", o.id, rh));
                }
            }
        }
        if hashes.is_empty() {
            pdf.draw_line("Aucune empreinte brute associée aux observations.", 11.0);
        } else {
            for h in hashes {
                pdf.draw_line(&h, 9.0);
            }
        }

        pdf.finish()
    }

    /// Génère un rapport DOCX complet sous forme de bytes.
    pub fn generate_docx(
        &self,
        store: &Store,
        opts: &ReportOptions,
    ) -> Result<Vec<u8>, ReportError> {
        use docx_rs::{
            Docx, Paragraph, Run, AlignmentType,
        };

        let entities = store.list_entities()?;
        let mut entity_obs: BTreeMap<String, Vec<Observation>> = BTreeMap::new();
        for e in &entities {
            let obs = store.get_observations_by_subject(&e.id)?;
            entity_obs.insert(e.id.clone(), obs);
        }

        let mut docx = Docx::new();

        // Titre
        docx = docx.add_paragraph(
            Paragraph::new()
                .add_run(Run::new().add_text(&opts.title).bold())
                .align(AlignmentType::Center),
        );

        // Métadonnées
        docx = docx.add_paragraph(
            Paragraph::new().add_run(Run::new().add_text(format!(
                "Dossier : {}  |  Opérateur : {}  |  Date : {}",
                opts.case_id,
                opts.operator,
                chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
            ))),
        );

        // Résumé
        docx = docx.add_paragraph(
            Paragraph::new().add_run(Run::new().add_text("Résumé").bold()),
        );
        let total_obs = entity_obs.values().map(|v| v.len()).sum::<usize>();
        docx = docx.add_paragraph(
            Paragraph::new().add_run(Run::new().add_text(format!(
                "{} entités, {} observations.",
                entities.len(),
                total_obs
            ))),
        );

        // Entités
        docx = docx.add_paragraph(
            Paragraph::new().add_run(Run::new().add_text("Entités et observations").bold()),
        );
        for e in &entities {
            docx = docx.add_paragraph(
                Paragraph::new().add_run(
                    Run::new().add_text(format!(
                        "{} ({:?}) — {}",
                        e.display_label, e.kind, e.canonical_value
                    )).bold(),
                ),
            );
            if let Some(obs) = entity_obs.get(&e.id) {
                for o in obs {
                    if !opts.include_inferred && o.provenance == Provenance::Inferred {
                        continue;
                    }
                    let val = serde_json::to_string(&o.value)
                        .unwrap_or_default()
                        .trim_matches('"')
                        .to_string();
                    let line = format!(
                        "• {} = {} (source: {}, confiance: {:?}, provenance: {:?})",
                        o.predicate, val, o.source, o.confidence, o.provenance
                    );
                    docx = docx.add_paragraph(
                        Paragraph::new().add_run(Run::new().add_text(line)),
                    );
                }
            }
        }

        // Journal d'audit
        docx = docx.add_paragraph(
            Paragraph::new().add_run(Run::new().add_text("Journal d'audit").bold()),
        );
        let audit_events = store.list_audit_events(&opts.case_id).unwrap_or_default();
        if audit_events.is_empty() {
            docx = docx.add_paragraph(
                Paragraph::new().add_run(Run::new().add_text("Aucun événement d'audit enregistré.")),
            );
        } else {
            for ev in &audit_events {
                let line = format!(
                    "[{}] {} — {} / {} (séq: {}, hash: {}…{})",
                    ev.timestamp,
                    ev.action,
                    ev.entity_kind,
                    ev.actor,
                    ev.sequence,
                    &ev.hash[..8.min(ev.hash.len())],
                    &ev.hash[ev.hash.len().saturating_sub(8)..]
                );
                docx = docx.add_paragraph(
                    Paragraph::new().add_run(Run::new().add_text(line)),
                );
            }
        }

        // Annexes
        docx = docx.add_paragraph(
            Paragraph::new().add_run(Run::new().add_text("Annexes — Empreintes de preuves").bold()),
        );
        let mut hashes: Vec<String> = Vec::new();
        for obs_list in entity_obs.values() {
            for o in obs_list {
                if let Some(rh) = &o.raw_hash {
                    hashes.push(format!("{} : {}", o.id, rh));
                }
            }
        }
        if hashes.is_empty() {
            docx = docx.add_paragraph(
                Paragraph::new().add_run(Run::new().add_text(
                    "Aucune empreinte brute associée aux observations.",
                )),
            );
        } else {
            for h in hashes {
                docx = docx.add_paragraph(
                    Paragraph::new().add_run(Run::new().add_text(h)),
                );
            }
        }

        docx = docx.add_paragraph(
            Paragraph::new().add_run(Run::new().add_text(
                "Rapport généré par SPECTRA. Les observations marquées « Inferred » ne sont pas des preuves.",
            )),
        );

        let xml = docx.build();
        let mut buf = std::io::Cursor::new(Vec::new());
        xml.pack(&mut buf)
            .map_err(|e| ReportError::Pdf(format!("erreur DOCX : {e}")))?;
        Ok(buf.into_inner())
    }

    fn to_markdown(
        &self,
        opts: &ReportOptions,
        entities: &[Entity],
        entity_obs: &BTreeMap<String, Vec<Observation>>,
    ) -> String {
        let mut md = String::new();
        md.push_str(&format!("# {}\n\n", opts.title));
        md.push_str(&format!("**Dossier :** {}\n\n", opts.case_id));
        md.push_str(&format!("**Opérateur :** {}\n\n", opts.operator));
        md.push_str(&format!(
            "**Date :** {}\n\n",
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        ));

        md.push_str("---\n\n");
        md.push_str(&format!(
            "## Résumé\n\n{} entités, {} observations.\n\n",
            entities.len(),
            entity_obs.values().map(|v| v.len()).sum::<usize>()
        ));

        md.push_str("## Entités\n\n");
        for e in entities {
            md.push_str(&format!(
                "### {} (`{}`)\n\n",
                e.display_label, e.canonical_value
            ));
            md.push_str(&format!("- **Type :** {:?}\n", e.kind));
            md.push_str(&format!("- **ID :** `{}`\n", e.id));
            if let Some(obs) = entity_obs.get(&e.id) {
                if !obs.is_empty() {
                    md.push_str("- **Observations :**\n");
                    for o in obs {
                        if !opts.include_inferred && o.provenance == Provenance::Inferred {
                            continue;
                        }
                        md.push_str(&format!(
                            "  - `{}` = `{}` (source: {}, confiance: {:?}, provenance: {:?})\n",
                            o.predicate,
                            serde_json::to_string(&o.value)
                                .unwrap_or_default()
                                .trim_matches('"'),
                            o.source,
                            o.confidence,
                            o.provenance
                        ));
                    }
                }
            }
            md.push('\n');
        }

        md.push_str("---\n\n");
        md.push_str(
            "*Rapport généré par SPECTRA. Les observations marquées « Inferred » ne sont pas des preuves.*\n",
        );
        md
    }

    fn to_html(
        &self,
        opts: &ReportOptions,
        entities: &[Entity],
        entity_obs: &BTreeMap<String, Vec<Observation>>,
    ) -> String {
        let mut html = String::new();
        html.push_str("<!DOCTYPE html>\n<html>\n<head>\n<meta charset=\"UTF-8\">\n");
        html.push_str(&format!(
            "<title>{} — Rapport SPECTRA</title>\n",
            html_escape(&opts.title)
        ));
        html.push_str(
            "<style>\n\
            body{font-family:system-ui,sans-serif;margin:2rem auto;max-width:900px;line-height:1.6;color:#222}\n\
            h1{border-bottom:2px solid #333;padding-bottom:.3rem}\n\
            h2{color:#444;border-bottom:1px solid #ccc}\n\
            table{border-collapse:collapse;width:100%;margin:1rem 0}\n\
            th,td{border:1px solid #ccc;padding:.5rem;text-align:left}\n\
            th{background:#f5f5f5}\n\
            .inferred{color:#888;font-style:italic}\n\
            .collected{color:#060;font-weight:700}\n\
            footer{margin-top:2rem;font-size:.85rem;color:#666;border-top:1px solid #ccc;padding-top:1rem}\n\
            </style>\n</head>\n<body>\n"
        );

        html.push_str(&format!("<h1>{}</h1>\n", html_escape(&opts.title)));
        html.push_str("<p><strong>Dossier :</strong> ");
        html.push_str(&html_escape(&opts.case_id));
        html.push_str("<br><strong>Opérateur :</strong> ");
        html.push_str(&html_escape(&opts.operator));
        html.push_str("<br><strong>Date :</strong> ");
        html.push_str(&chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string());
        html.push_str("</p>\n");

        html.push_str("<h2>Résumé</h2>\n");
        html.push_str("<p>");
        html.push_str(&format!(
            "{} entités, {} observations.",
            entities.len(),
            entity_obs.values().map(|v| v.len()).sum::<usize>()
        ));
        html.push_str("</p>\n");

        html.push_str("<h2>Entités</h2>\n<table>\n");
        html.push_str(
            "<tr><th>Label</th><th>Type</th><th>Valeur</th><th>Observations</th></tr>\n"
        );
        for e in entities {
            let obs_list = entity_obs.get(&e.id).cloned().unwrap_or_default();
            let mut obs_cells = String::new();
            for o in &obs_list {
                if !opts.include_inferred && o.provenance == Provenance::Inferred {
                    continue;
                }
                let cls = match o.provenance {
                    Provenance::Inferred => "inferred",
                    _ => "collected",
                };
                let val = serde_json::to_string(&o.value)
                    .unwrap_or_default()
                    .trim_matches('"')
                    .to_string();
                obs_cells.push_str(&format!(
                    "<span class=\"{}\">{} = {} ({})</span><br>",
                    cls,
                    html_escape(&o.predicate),
                    html_escape(&val),
                    html_escape(&o.source)
                ));
            }
            html.push_str(&format!(
                "<tr><td>{}</td><td>{:?}</td><td>{}</td><td>{}</td></tr>\n",
                html_escape(&e.display_label),
                e.kind,
                html_escape(&e.canonical_value),
                obs_cells
            ));
        }
        html.push_str("</table>\n");

        html.push_str("<footer>\n");
        html.push_str(
            "Rapport généré par SPECTRA. Les observations marquées « Inferred » ne sont pas des preuves.\n",
        );
        html.push_str("</footer>\n</body>\n</html>");
        html
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

// ---------------------------------------------------------------------------
// PDF helpers (printpdf)
// ---------------------------------------------------------------------------

use printpdf::{BuiltinFont, Color, IndirectFontRef, Mm, PdfDocument, Rgb};
use printpdf::indices::{PdfLayerIndex, PdfPageIndex};

struct PdfReport {
    doc: printpdf::PdfDocumentReference,
    current_page: PdfPageIndex,
    current_layer: PdfLayerIndex,
    font: IndirectFontRef,
    y: f64,
    page_width: f64,
    page_height: f64,
    margin: f64,
}

impl PdfReport {
    fn new() -> Result<Self, printpdf::Error> {
        let (doc, page1, layer1) = PdfDocument::new(
            "SPECTRA Report",
            Mm(210.0),
            Mm(297.0),
            "Layer 1",
        );
        let font = doc.add_builtin_font(BuiltinFont::Helvetica)?;
        Ok(Self {
            doc,
            current_page: page1,
            current_layer: layer1,
            font,
            y: 280.0,
            page_width: 210.0,
            page_height: 297.0,
            margin: 15.0,
        })
    }

    fn current_layer(&self) -> printpdf::PdfLayerReference {
        self.doc.get_page(self.current_page).get_layer(self.current_layer)
    }

    fn new_page(&mut self) {
        let (page, layer) = self.doc.add_page(
            Mm(self.page_width),
            Mm(self.page_height),
            "Layer 1",
        );
        self.current_page = page;
        self.current_layer = layer;
        self.y = self.page_height - self.margin;
    }

    fn ensure_space(&mut self, needed: f64) {
        if self.y - needed < self.margin {
            self.new_page();
        }
    }

    fn draw_title(&mut self, text: &str, size: f64) {
        self.ensure_space(size + 4.0);
        let layer = self.current_layer();
        layer.set_fill_color(Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None)));
        layer.use_text(text, size, Mm(self.margin), Mm(self.y), &self.font);
        self.y -= size + 6.0;
    }

    fn draw_line(&mut self, text: &str, size: f64) {
        self.ensure_space(size + 2.0);
        let layer = self.current_layer();
        layer.set_fill_color(Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None)));
        // Tronquer si la ligne dépasse ~90 caractères (approximation large pour éviter les débordements)
        let display = if text.len() > 180 {
            format!("{}…", &text[..180])
        } else {
            text.to_string()
        };
        layer.use_text(display, size, Mm(self.margin), Mm(self.y), &self.font);
        self.y -= size + 2.0;
    }

    fn draw_colored_line(&mut self, text: &str, size: f64, color: Color) {
        self.ensure_space(size + 2.0);
        let layer = self.current_layer();
        layer.set_fill_color(color);
        let display = if text.len() > 180 {
            format!("{}…", &text[..180])
        } else {
            text.to_string()
        };
        layer.use_text(display, size, Mm(self.margin), Mm(self.y), &self.font);
        self.y -= size + 2.0;
    }

    fn advance(&mut self, amount: f64) {
        self.y -= amount;
    }

    fn finish(self) -> Result<Vec<u8>, ReportError> {
        let mut buf = Vec::new();
        {
            let mut writer = BufWriter::new(&mut buf);
            self.doc.save(&mut writer)?;
        }
        Ok(buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use spectra_core::{
        AdmiraltyCode, Entity, EntityKind, Observation, PropertyValue, Provenance,
    };
    use std::collections::BTreeMap;

    fn dummy_entity(id: &str) -> Entity {
        Entity {
            id: id.to_string(),
            kind: EntityKind::Username,
            canonical_value: id.to_string(),
            display_label: id.to_string(),
            properties: BTreeMap::new(),
            created_at: "2026-01-01T00:00:00Z".to_string(),
            merged_from: vec![],
        }
    }

    fn dummy_obs(subject: &str, predicate: &str, value: &str) -> Observation {
        Observation {
            id: format!("obs-{subject}-{predicate}"),
            subject: subject.to_string(),
            predicate: predicate.to_string(),
            value: PropertyValue::String(value.to_string()),
            source: "test".to_string(),
            method: "TEST".to_string(),
            observed_at: "2026-01-01T00:00:00Z".to_string(),
            valid_from: None,
            valid_to: None,
            confidence: AdmiraltyCode::from_str("A1").unwrap(),
            provenance: Provenance::Collected,
            raw_hash: None,
            operator: "test".to_string(),
        }
    }

    #[test]
    fn markdown_report_smoke() {
        let store = Store::open_in_memory().unwrap();
        let e = dummy_entity("e1");
        store.insert_entity(&e).unwrap();
        store.insert_observation(&dummy_obs("e1", "bio", "Rust dev")).unwrap();

        let engine = ReportEngine::new();
        let opts = ReportOptions {
            title: "Rapport test".to_string(),
            case_id: "CASE-01".to_string(),
            operator: "Alice".to_string(),
            include_inferred: false,
            format: ReportFormat::Markdown,
        };
        let md = engine.generate(&store, &opts).unwrap();
        assert!(md.contains("# Rapport test"));
        assert!(md.contains("Rust dev"));
        assert!(md.contains("CASE-01"));
    }

    #[test]
    fn html_report_smoke() {
        let store = Store::open_in_memory().unwrap();
        let e = dummy_entity("e1");
        store.insert_entity(&e).unwrap();
        store.insert_observation(&dummy_obs("e1", "bio", "Rust dev")).unwrap();

        let engine = ReportEngine::new();
        let opts = ReportOptions {
            title: "Rapport test".to_string(),
            case_id: "CASE-01".to_string(),
            operator: "Alice".to_string(),
            include_inferred: false,
            format: ReportFormat::Html,
        };
        let html = engine.generate(&store, &opts).unwrap();
        assert!(html.starts_with("<!DOCTYPE html>"));
        assert!(html.contains("Rust dev"));
        assert!(html.contains("SPECTRA"));
    }

    #[test]
    fn pdf_report_smoke() {
        let store = Store::open_in_memory().unwrap();
        let e = dummy_entity("e1");
        store.insert_entity(&e).unwrap();
        let mut obs = dummy_obs("e1", "bio", "Rust dev");
        obs.raw_hash = Some("abc123".to_string());
        store.insert_observation(&obs).unwrap();

        let engine = ReportEngine::new();
        let opts = ReportOptions {
            title: "Rapport test PDF".to_string(),
            case_id: "CASE-01".to_string(),
            operator: "Alice".to_string(),
            include_inferred: false,
            format: ReportFormat::Pdf,
        };
        let pdf = engine.generate_pdf(&store, &opts).unwrap();
        assert!(!pdf.is_empty());
        assert!(pdf.starts_with(b"%PDF"));
    }

    #[test]
    fn docx_report_smoke() {
        let store = Store::open_in_memory().unwrap();
        let e = dummy_entity("e1");
        store.insert_entity(&e).unwrap();
        let mut obs = dummy_obs("e1", "bio", "Rust dev");
        obs.raw_hash = Some("abc123".to_string());
        store.insert_observation(&obs).unwrap();

        let engine = ReportEngine::new();
        let opts = ReportOptions {
            title: "Rapport test DOCX".to_string(),
            case_id: "CASE-01".to_string(),
            operator: "Alice".to_string(),
            include_inferred: false,
            format: ReportFormat::Docx,
        };
        let docx = engine.generate_docx(&store, &opts).unwrap();
        assert!(!docx.is_empty());
        // Un fichier DOCX est un ZIP dont la signature commence par PK
        assert!(docx.starts_with(b"PK"));
    }
}
