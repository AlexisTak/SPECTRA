//! Schéma de sonde interne — cœur du système OSINT.
//!
//! Une sonde = un template de requête + des assertions de décision.
//! Ajouter un nouvel outil = un convertisseur de ~150 lignes vers ce schéma.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Identifiant unique d'une sonde : "wmn:github", "sherlock:reddit"
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProbeId(pub String);

/// Catégorie de site
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    Social,
    Dev,
    Gaming,
    Forum,
    Adult,
    Finance,
    Shopping,
    News,
    Education,
    Music,
    Video,
    Dating,
    Other,
}

/// Type de sélecteur
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SelectorKind {
    Username,
    Email,
    Phone,
}

/// Template de requête HTTP
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestTemplate {
    /// Méthode HTTP
    pub method: Method,
    /// URL avec substitution {selector}
    pub url: String,
    /// Headers HTTP
    #[serde(default)]
    pub headers: HashMap<String, String>,
    /// Corps de la requête (JSON ou form)
    pub body: Option<BodyTemplate>,
    /// Suivre les redirections
    #[serde(default = "default_true")]
    pub follow_redirects: bool,
    /// Timeout de la requête
    #[serde(default, with = "humantime_serde")]
    pub timeout: Duration,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Method {
    Get,
    Post,
    Head,
    Options,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BodyTemplate {
    /// JSON avec substitution {selector}
    Json(String),
    /// Form-encoded
    Form(HashMap<String, String>),
}

/// Règles de décision — ordre d'évaluation strict
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionRules {
    /// Conditions d'existence (OU logique)
    #[serde(default)]
    pub exists_if: Vec<Assertion>,
    /// Conditions d'absence (OU logique)
    #[serde(default)]
    pub missing_if: Vec<Assertion>,
    /// Conditions de blocage (Cloudflare, captcha, rate limit)
    #[serde(default)]
    pub blocked_if: Vec<Assertion>,
}

/// Assertion pour la décision
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Assertion {
    /// Code HTTP exact
    StatusCode(u16),
    /// Code dans un intervalle
    StatusIn { min: u16, max: u16 },
    /// Corps contient une chaîne
    BodyContains(String),
    /// Corps ne contient pas une chaîne
    BodyNotContains(String),
    /// Corps correspond à une regex
    BodyMatches(String),
    /// URL finale contient une chaîne
    FinalUrlContains(String),
    /// URL finale égale une valeur
    FinalUrlEquals(String),
    /// Header présent
    HeaderPresent(String),
    /// Valeur JSON à un pointeur
    JsonPointerEquals { pointer: String, value: serde_json::Value },
    /// Longueur de contenu > N
    ContentLengthGreaterThan(usize),
}

/// Extraction de métadonnées si le compte existe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Extraction {
    /// Nom de la métadonnée
    pub name: String,
    /// Comment l'extraire (XPath, CSS, JSON pointer)
    pub selector: String,
    /// Type de donnée
    pub kind: ExtractionKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExtractionKind {
    Text,
    Attribute { attr: String },
    JsonPointer { pointer: String },
    Regex { pattern: String },
}

/// Qualité mesurée d'une sonde
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProbeQuality {
    /// Taux de succès du contrôle (0.0 - 1.0)
    pub control_success_rate: f64,
    /// Taux de blocage (0.0 - 1.0)
    pub block_rate: f64,
    /// Latence médiane en ms
    pub median_latency_ms: u64,
    /// Dernière vérification réussie
    pub last_success: Option<chrono::DateTime<chrono::Utc>>,
    /// Niveau de confiance
    pub confidence: ConfidenceLevel,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfidenceLevel {
    #[default]
    Unknown,
    High,
    Medium,
    Low,
    Degraded,
}

/// Provenance de la sonde
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSource {
    /// Source : "wmn", "sherlock", "maigret", "holehe", "manual"
    pub source: String,
    /// Licence : "CC-BY-SA-4.0", "MIT", "GPL-3.0", "internal"
    pub license: String,
    /// URL du fichier source
    pub source_url: Option<String>,
    /// Hash du fichier source (pour pinning)
    pub source_hash: Option<String>,
}

/// Une sonde complète
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Probe {
    pub id: ProbeId,
    pub site_name: String,
    pub category: Category,
    pub selector_kind: SelectorKind,
    pub request: RequestTemplate,
    pub decision: DecisionRules,
    #[serde(default)]
    pub extract: Vec<Extraction>,
    #[serde(default)]
    pub quality: ProbeQuality,
    pub source: DataSource,
}

/// Résultat d'une sonde
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum ProbeOutcome {
    /// Compte trouvé
    Exists {
        evidence: Evidence,
        extracted: HashMap<String, String>,
    },
    /// Compte inexistant
    Missing,
    /// Bloqué (Cloudflare, captcha, rate limit)
    Blocked { reason: BlockReason },
    /// Indéterminé — ne pas trancher
    Indeterminate { reason: String },
    /// Erreur technique
    Error(ProbeError),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlockReason {
    Cloudflare,
    Captcha,
    RateLimit,
    Waf,
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProbeError {
    Network(String),
    Timeout,
    InvalidUrl(String),
    ParseError(String),
    Internal(String),
}

/// Preuve de la décision — pour la chaîne de possession
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    /// Assertion déclenchée
    pub triggered_assertion: Option<String>,
    /// Code HTTP
    pub status_code: u16,
    /// URL finale (après redirections)
    pub final_url: String,
    /// Hash BLAKE3 du corps de réponse
    pub body_hash: String,
    /// Corps tronqué (256 octets max pour le debug)
    pub body_preview: Option<String>,
    /// Timestamp de la requête
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Résultat complet d'une exécution de sonde
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeResult {
    pub probe_id: ProbeId,
    pub selector: String,
    pub outcome: ProbeOutcome,
    /// Temps d'exécution en ms
    pub elapsed_ms: u64,
}
