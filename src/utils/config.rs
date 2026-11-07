use crate::types::{AppConfig, SyncConfig};
use chrono::Utc;
use directories::ProjectDirs;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use uuid::uuid;

fn get_config_file_path() -> Option<PathBuf> {
    // Qualifiers: "com", organization: "MyOrg", application: "MyCoolApp"
    // On Windows, this resolves to: C:\Users\<User>\AppData\Roaming\Randydator\Syncee\config.json
    if let Some(proj_dirs) = ProjectDirs::from("com", "Randydator", "Syncee") {
        let config_dir = proj_dirs.config_dir();

        if !config_dir.exists() {
            fs::create_dir_all(config_dir).ok()?;
        }

        Some(config_dir.join("config.json"))
    } else {
        None
    }
}

fn write_updated_app_config(app_config: &AppConfig) -> Result<(), String> {
    let path = get_config_file_path().ok_or("Could not determine configuration directory!")?;

    let json_data = serde_json::to_string_pretty(app_config)
        .map_err(|e| format!("Failed to serialize AppConfig to JSON: {}", e))?;

    fs::write(&path, json_data)
        .map_err(|e| format!("Failed to write config file to {:?}: {}", path, e))?;

    Ok(())
}

pub fn ensure_app_config_initialized() -> Option<AppConfig> {
    let path = get_config_file_path()?;
    let default_config = AppConfig {
        dark_mode: true,
        sync_configs: HashMap::new(),
    };

    if path.exists() {
        match fs::read_to_string(&path) {
            Ok(content) => match serde_json::from_str::<AppConfig>(&content) {
                Ok(config) => Some(config),
                Err(e) => {
                    log::warn!("Failed to parse config.json. Creating a fresh one: {}", e);
                    let _ = write_updated_app_config(&default_config);
                    Some(default_config)
                }
            },
            Err(e) => {
                log::error!("Failed to read config file: {}", e);
                None
            }
        }
    } else {
        let _ = write_updated_app_config(&default_config);
        Some(default_config)
    }
}

pub fn create_or_update_sync_config(sync_config: SyncConfig) {
    let mut app_config = match ensure_app_config_initialized() {
        Some(config) => config,
        None => {
            log::error!("Could not initialize app configuration!");
            return;
        }
    };

    let config_id = sync_config.id;

    // Insert into the HashMap. If the ID exists, it updates it; otherwise, it creates it.
    app_config.sync_configs.insert(config_id, sync_config);

    // Save changes back to disk
    match write_updated_app_config(&app_config) {
        Ok(_) => {
            log::info!(
                "Successfully saved sync config (ID: {}) to configuration file.",
                config_id
            );
        }
        Err(e) => {
            log::error!("{}", e);
        }
    }
}

pub fn get_sync_configs() -> Vec<SyncConfig> {
    let app_config = match ensure_app_config_initialized() {
        Some(config) => config,
        None => {
            log::error!("Could not initialize app configuration to fetch sync configs!");
            return Vec::new();
        }
    };

    // Extract all values from the HashMap and clone them into a Vec
    app_config.sync_configs.values().cloned().collect()
}

pub fn write_test_data() {
    // Create an instance of your SyncConfig struct with sample data
    let sample_config = SyncConfig {
        id: uuid!("9254c298-2aa8-4c7b-bfdb-c776847a8887"),
        title: "My Sync Task".to_string(),
        description: "Synchronizing local directory with remote storage".to_string(),
        last_synced: Utc::now(),
    };

    create_or_update_sync_config(sample_config);
    log::info!("Test sync config created and saved successfully.");
}
