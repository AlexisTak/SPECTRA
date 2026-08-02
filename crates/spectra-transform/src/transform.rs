//! Trait `Transform` et types associés.

use serde::{Deserialize, Serialize};
use spectra_core::{Entity, EntityId, Observation};
use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

/// Entrée d'un transform : une entité du graphe, avec ses propriétés et le
/// contexte d'exécution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformInput {
    /// Entité sur laquelle le transform opère.
    pub entity: Entity,
    /// Propriétés additionnelles passées par l'orchestrateur (paramètres de
    /// l'analyste, options du transform).
    pub params: BTreeMap<String, serde_json::Value>,
}

/// Sortie d'un transform : nouvelles entités, observations et relations créées.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TransformOutput {
    /// Nouvelles entités découvertes par le transform.
    pub entities: Vec<Entity>,
    /// Observations produites sur l'entité d'entrée ou les nouvelles entités.
    pub observations: Vec<Observation>,
    /// Relations entre l'entité d'entrée et les nouvelles entités.
    pub relations: Vec<Relation>,
}

/// Une relation orientée entre deux entités.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relation {
    /// Entité d'origine du lien.
    pub source: EntityId,
    /// Entité cible du lien.
    pub target: EntityId,
    /// Type sémantique de la relation : `"resolved_to"`, `"hosted_on"`,
    /// `"registered_by"`, etc.
    pub kind: String,
    /// Propriétés qualifiant la relation.
    pub properties: BTreeMap<String, serde_json::Value>,
}

/// Contexte d'exécution passé à chaque transform.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformContext {
    /// Identifiant du dossier d'enquête.
    pub case_id: String,
    /// Profil réseau actif : `"direct"`, `"proxy"`, `"tor"`.
    pub network_profile: String,
    /// User-Agent à utiliser pour les requêtes HTTP.
    pub user_agent: String,
    /// Opérateur à l'origine de la collecte.
    pub operator: String,
}

/// Erreur produite par un transform.
#[derive(Debug, Clone, thiserror::Error)]
pub enum TransformError {
    /// Le type d'entité d'entrée n'est pas supporté par ce transform.
    #[error("type d'entité non supporté : {0}")]
    UnsupportedEntityType(String),
    /// Le transform a échoué à cause d'une erreur réseau.
    #[error("erreur réseau : {0}")]
    NetworkError(String),
    /// Le transform a dépassé son délai d'attente.
    #[error("délai d'attente dépassé")]
    Timeout,
    /// Le transform a été annulé par l'utilisateur.
    #[error("annulé")]
    Cancelled,
    /// Le transform a violé sa sandbox (tentative d'accès hors allowlist).
    #[error("violation de sandbox : {0}")]
    SandboxViolation(String),
    /// Erreur interne du transform.
    #[error("erreur interne : {0}")]
    Internal(String),
}

/// Contrat commun à tous les transforms — natifs ou WASM.
///
/// Les implémentations natives implémentent directement ce trait. Les plugins
/// WASM sont wrappés par une implémentation dans `spectra-plugins` qui traduit
/// les appels host functions en exécutions du module.
pub trait Transform: Send + Sync {
    /// Identifiant unique du transform (ex: `dns:a`, `whois:rir`).
    fn id(&self) -> &str;

    /// Nom lisible pour l'UI.
    fn display_name(&self) -> &str;

    /// Types d'entités d'entrée acceptés.
    fn input_kinds(&self) -> &[spectra_core::EntityKind];

    /// Exécute le transform de manière asynchrone.
    fn execute(
        &self,
        ctx: TransformContext,
        input: TransformInput,
    ) -> Pin<Box<dyn Future<Output = Result<TransformOutput, TransformError>> + Send + '_>>;
}

/// Wrapper pour permettre l'utilisation de `dyn Transform` dans des collections.
pub type BoxedTransform = Box<dyn Transform>;
