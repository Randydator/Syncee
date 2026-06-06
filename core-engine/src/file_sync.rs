use std::path::PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncJob {
    pub name: String,
    pub enabled: bool,
    pub interval_hours: u64,
    pub last_synced: Option<String>,
    pub source_dest_pairs: Vec<(PathBuf, PathBuf)>,
    pub musicbee_root: PathBuf,
    pub target_playlists: Vec<String>,
    pub target_folders: Vec<String>,
    pub m3u_replacements: Vec<(String, String)>,
    pub mirror_sync: bool,
}

impl SyncJob {
    // The actual logic that runs the sync live lives right next to the data
    pub async fn run_sync(&mut self) -> Result<(), String> {
        println!("Starting sync for job: {}", self.name);
        // your walkdir, file copying, and playlist generation logic goes here
        Ok(())
    }
}