//! Wrapper `Transform` pour les plugins WASM Extism.

use extism::{
    Manifest as ExtismManifest, PluginBuilder, UserData, ValType, Wasm,
};
use spectra_core::EntityKind;
use spectra_transform::{
    Transform, TransformContext, TransformError, TransformInput, TransformOutput,
};
use std::future::Future;
use std::pin::Pin;

use crate::host::{spectra_http_get, AllowlistUserData};
use crate::manifest::PluginManifest;

/// Un transform qui délègue son exécution à un plugin WASM Extism.
pub struct PluginTransform {
    manifest: PluginManifest,
    wasm_bytes: Vec<u8>,
}

impl PluginTransform {
    /// Crée un nouveau PluginTransform à partir du manifeste et des bytes WASM.
    pub fn new(manifest: PluginManifest, wasm_bytes: Vec<u8>) -> Self {
        Self {
            manifest,
            wasm_bytes,
        }
    }

    /// Vérifie qu'un domaine est autorisé par ce plugin.
    pub fn allows_domain(&self, domain: &str) -> bool {
        self.manifest.allows_domain(domain)
    }

    /// Exécute un export arbitraire du plugin (par défaut `"transform"`) en
    /// injectant la host function `spectra_http_get` qui applique l'allowlist.
    pub fn execute_export(
        &self,
        export: &str,
        ctx: TransformContext,
        input: TransformInput,
    ) -> Result<TransformOutput, TransformError> {
        let wasm = Wasm::data(self.wasm_bytes.clone());
        let extism_manifest = ExtismManifest::new([wasm]);
        let allowlist = AllowlistUserData::new(self.manifest.permissions.network.allowlist.clone());
        let user_data = UserData::new(allowlist);

        let mut plugin = PluginBuilder::new(extism_manifest)
            .with_wasi(true)
            .with_function(
                "spectra_http_get",
                [ValType::I64],
                [ValType::I64],
                user_data,
                spectra_http_get,
            )
            .build()
            .map_err(|e| TransformError::Internal(format!("Extism init: {e}")))?;

        let payload = serde_json::json!({
            "context": ctx,
            "input": input,
        });
        let payload_json = serde_json::to_vec(&payload)
            .map_err(|e| TransformError::Internal(format!("serialize: {e}")))?;

        let output_bytes = plugin
            .call(export, &payload_json)
            .map_err(|e| TransformError::Internal(format!("WASM call: {e}")))?;

        let output: TransformOutput = serde_json::from_slice(output_bytes)
            .map_err(|e| TransformError::Internal(format!("deserialize: {e}")))?;

        Ok(output)
    }
}

impl Transform for PluginTransform {
    fn id(&self) -> &str {
        &self.manifest.meta.id
    }

    fn display_name(&self) -> &str {
        &self.manifest.meta.name
    }

    fn input_kinds(&self) -> &[spectra_core::EntityKind] {
        self.manifest
            .transforms
            .first()
            .map(|t| t.input_kinds.as_slice())
            .unwrap_or(&[])
    }

    fn execute(
        &self,
        ctx: TransformContext,
        input: TransformInput,
    ) -> Pin<Box<dyn Future<Output = Result<TransformOutput, TransformError>> + Send + '_>> {
        Box::pin(async move { self.execute_export("transform", ctx, input) })
    }
}
