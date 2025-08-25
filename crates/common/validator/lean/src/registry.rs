use std::{fs, path::Path};

use serde::{Deserialize, Serialize};

/// YAML structure representing the validator registry file
#[derive(Debug, Deserialize, Serialize)]
pub struct ValidatorRegistryYaml {
    pub validators: Vec<ValidatorEntry>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ValidatorEntry {
    pub validator_id: u64,
    pub keystore_path: String,
}

// TODO: We need to replace this after PQC integration.
// For now, we only need ID for keystore.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeanKeystore {
    pub validator_id: u64,
}

/// Load validator registry from YAML file
pub fn load_validator_registry<P: AsRef<Path>>(path: P) -> anyhow::Result<Vec<LeanKeystore>> {
    let content = fs::read_to_string(&path).map_err(|err| {
        anyhow::anyhow!(
            "Failed to read validator registry file {:?}: {err}",
            path.as_ref(),
        )
    })?;

    let registry = serde_yaml::from_str::<ValidatorRegistryYaml>(&content)
        .map_err(|err| anyhow::anyhow!("Failed to parse validator registry YAML: {}", err))?;

    Ok(registry
        .validators
        .into_iter()
        .map(|entry| LeanKeystore {
            validator_id: entry.validator_id,
        })
        .collect())
}
