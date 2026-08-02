//! Registre de plugins — chargement, signature, stockage.

use crate::manifest::PluginManifest;
use crate::transform::PluginTransform;
use spectra_transform::Transform;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use std::collections::HashMap;
use std::path::Path;

/// Registre local de plugins WASM.
pub struct PluginRegistry {
    plugins: HashMap<String, PluginTransform>,
}

impl PluginRegistry {
    /// Crée un registre vide.
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
        }
    }

    /// Charge un plugin depuis le filesystem : manifeste TOML + fichier WASM
    /// + signature.
    ///
    /// # Arguments
    ///
    /// * `manifest_path` — chemin vers `manifest.toml`
    /// * `wasm_path` — chemin vers `plugin.wasm`
    /// * `signature_path` — chemin vers la signature Ed25519 (fichier binaire)
    /// * `public_key` — clé publique attendue (hex, 64 caractères)
    ///
    /// # Erreurs
    ///
    /// Retourne une erreur si la signature est invalide, si le manifeste est
    /// malformé, ou si le WASM ne charge pas.
    pub fn install(
        &mut self,
        manifest_path: &Path,
        wasm_path: &Path,
        signature_path: &Path,
        public_key_hex: &str,
    ) -> Result<(), PluginInstallError> {
        let manifest_toml = std::fs::read_to_string(manifest_path)
            .map_err(|e| PluginInstallError::Io(format!("manifest: {e}")))?;
        let manifest = PluginManifest::from_toml(&manifest_toml)
            .map_err(|e| PluginInstallError::Manifest(format!("{e}")))?;

        let wasm_bytes = std::fs::read(wasm_path)
            .map_err(|e| PluginInstallError::Io(format!("wasm: {e}")))?;
        let signature_bytes = std::fs::read(signature_path)
            .map_err(|e| PluginInstallError::Io(format!("signature: {e}")))?;

        // Vérification Ed25519.
        let public_key_bytes = hex::decode(public_key_hex)
            .map_err(|e| PluginInstallError::InvalidKey(format!("{e}")))?;
        let verifying_key = VerifyingKey::from_bytes(
            &public_key_bytes
                .try_into()
                .map_err(|_| PluginInstallError::InvalidKey("taille".to_string()))?,
        )
        .map_err(|e| PluginInstallError::InvalidKey(format!("{e}")))?;

        let signature = Signature::from_slice(&signature_bytes)
            .map_err(|e| PluginInstallError::InvalidSignature(format!("{e}")))?;

        verifying_key
            .verify(&wasm_bytes, &signature)
            .map_err(|_| PluginInstallError::InvalidSignature("vérification échouée".to_string()))?;

        let pt = PluginTransform::new(manifest, wasm_bytes);
        let id = pt.id().to_string();
        self.plugins.insert(id, pt);
        Ok(())
    }

    /// Récupère un transform chargé par son identifiant.
    pub fn get(&self, id: &str) -> Option<&PluginTransform> {
        self.plugins.get(id)
    }

    /// Liste les identifiants des plugins chargés.
    pub fn list(&self) -> Vec<&str> {
        self.plugins.keys().map(|s| s.as_str()).collect()
    }
}

/// Erreur d'installation d'un plugin.
#[derive(Debug, thiserror::Error)]
pub enum PluginInstallError {
    /// Lecture ou écriture impossible dans le registre local.
    #[error("erreur d'entrée/sortie : {0}")]
    Io(String),
    /// Manifeste TOML absent, illisible ou incomplet.
    #[error("manifeste malformé : {0}")]
    Manifest(String),
    /// Clé publique Ed25519 malformée.
    #[error("clé publique invalide : {0}")]
    InvalidKey(String),
    /// Signature du module WASM non vérifiable avec la clé attendue.
    #[error("signature invalide : {0}")]
    InvalidSignature(String),
}
