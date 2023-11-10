use std::{fs, fs::File, path::PathBuf, str};

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

/// Application state
#[derive(Deserialize, Serialize)]
pub struct Application {
    pub config_path: PathBuf,
    pub metadata_path: PathBuf,
    pub node_address: String,
    pub contract_address: Option<String>,
    pub vote_pk: PathBuf,
    pub verdict_none_pk: PathBuf,
    pub verdict_negative_pk: PathBuf,
    pub verdict_positive_pk: PathBuf,
}

impl Default for Application {
    fn default() -> Self {
        Self {
            config_path: ".bright_disputes_config.json".into(),
            metadata_path: "../contract/target/ink/bright_disputes.json".into(),
            node_address: "ws://127.0.0.1:9944".into(),
            contract_address: None,
            vote_pk: "../scripts/docker/keys/vote.groth16.pk.bytes".into(),
            verdict_none_pk: "../scripts/docker/keys/verdict_none.groth16.pk.bytes".into(),
            verdict_negative_pk: "../scripts/docker/keys/verdict_negative.groth16.pk.bytes".into(),
            verdict_positive_pk: "../scripts/docker/keys/verdict_positive.groth16.pk.bytes".into(),            
        }
    }
}

impl Application {
    /// Load or create the application state from 'path'
    pub fn load_or_create(path: &PathBuf) -> Result<Self> {
        if path.exists() {
            let content = fs::read(path).map_err(|e| anyhow!("Failed to load file: {e}"))?;

            serde_json::from_slice::<Application>(&content)
                .map_err(|e| anyhow!("Failed to deserialize: {e}"))
        } else {
            let app = Application::default();
            Self::save(path, &app)?;
            Ok(app)
        }
    }

    /// Store the application state at 'path'
    pub fn save(path: &PathBuf, app: &Application) -> Result<()> {
        let content =
            serde_json::to_string_pretty(app).map_err(|e| anyhow!("Failed to serialize: {e}"))?;

        if !path.exists() {
            File::create(path).map_err(|e| anyhow!("Failed to create {path:?}: {e}"))?;
        }
        fs::write(path, content).map_err(|e| anyhow!("Failed to save application config: {e}"))
    }
}
