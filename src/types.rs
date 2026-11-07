use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SyncConfig {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub last_synced: DateTime<Utc>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct AppConfig {
    pub dark_mode: bool,
    pub sync_configs: HashMap<Uuid, SyncConfig>,
}
