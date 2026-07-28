//! Convertisseur WhatsMyName → schéma interne.
//!
//! WhatsMyName fournit wmn-data.json avec un schéma documenté dans
//! wmn-data-schema.json. Ce convertisseur lit le fichier et produit des sondes.

use crate::report::ConversionReport;
use serde::Deserialize;
use spectra_probe::*;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
struct WmnData {
    sites: Vec<WmnSite>,
}

#[derive(Debug, Deserialize)]
struct WmnSite {
    name: String,
    uri_check: String,
    #[serde(default)]
    uri: Option<String>,
    #[serde(default)]
    category: Option<String>,
    #[serde(default)]
    e_code: Option<u16>,
    #[serde(default)]
    e_string: Option<String>,
    #[serde(default)]
    m_string: Option<String>,
    #[serde(default)]
    regex_check: Option<String>,
    #[serde(default)]
    url: Option<String>,
    #[serde(default)]
    known: Option<Vec<String>>,
}

pub fn convert(content: &str, source_hash: &str) -> Result<(Vec<Probe>, ConversionReport), Box<dyn std::error::Error>> {
    let data: WmnData = serde_json::from_str(content)?;
    let mut probes = Vec::new();
    let mut report = ConversionReport::new();

    for site in data.sites {
        match convert_site(&site, source_hash) {
            Ok(probe) => probes.push(probe),
            Err(e) => report.reject("wmn", site.name, e.to_string()),
        }
    }

    Ok((probes, report))
}

fn convert_site(site: &WmnSite, source_hash: &str) -> Result<Probe, String> {
    // Catégorie
    let category = parse_category(site.category.as_deref());

    // Règles de décision
    let mut exists_if = Vec::new();
    let mut missing_if = Vec::new();
    let mut blocked_if = Vec::new();

    // Code HTTP attendu pour "existe"
    if let Some(code) = site.e_code {
        if code == 200 {
            exists_if.push(Assertion::StatusCode(200));
        } else {
            // Code spécifique = existe
            exists_if.push(Assertion::StatusCode(code));
        }
    } else {
        // Par défaut, 200 = existe
        exists_if.push(Assertion::StatusCode(200));
    }

    // String dans le corps = manquant
    if let Some(s) = &site.e_string {
        missing_if.push(Assertion::BodyContains(s.clone()));
    }

    // m_string = string attendu si manquant (variante)
    if let Some(s) = &site.m_string {
        missing_if.push(Assertion::BodyContains(s.clone()));
    }

    // Blocked: Cloudflare 403
    blocked_if.push(Assertion::StatusCode(403));
    blocked_if.push(Assertion::StatusCode(429));

    // URL de la sonde
    let url = site.uri_check.replace("{account}", "{selector}");

    Ok(Probe {
        id: ProbeId(format!("wmn:{}", normalize_name(&site.name))),
        site_name: site.name.clone(),
        category,
        selector_kind: SelectorKind::Username,
        request: RequestTemplate {
            method: Method::Get,
            url,
            headers: HashMap::new(),
            body: None,
            follow_redirects: true,
            timeout: std::time::Duration::from_secs(30),
        },
        decision: DecisionRules {
            exists_if,
            missing_if,
            blocked_if,
        },
        extract: Vec::new(),
        quality: ProbeQuality::default(),
        source: DataSource {
            source: "wmn".to_string(),
            license: "CC-BY-SA-4.0".to_string(),
            source_url: Some("https://github.com/WebBreacher/WhatsMyName".to_string()),
            source_hash: Some(source_hash.to_string()),
        },
    })
}

fn parse_category(cat: Option<&str>) -> Category {
    match cat {
        Some("social") => Category::Social,
        Some("dev") => Category::Dev,
        Some("gaming") => Category::Gaming,
        Some("forum") => Category::Forum,
        Some("finance") => Category::Finance,
        Some("shopping") => Category::Shopping,
        Some("news") => Category::News,
        Some("education") => Category::Education,
        Some("music") => Category::Music,
        Some("video") => Category::Video,
        Some("dating") => Category::Dating,
        Some("adult") => Category::Adult,
        _ => Category::Other,
    }
}

fn normalize_name(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
        .collect()
}
