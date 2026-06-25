use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CoreEvent {
    JobStarted {
        name: String,
        timestamp: String,
        total_files: usize,
    },
    
    ProgressUpdate {
        job_name: String,
        files_processed: usize,
        total_files: usize,
    },
    
    TrackSyncError {
        job_name: String,
        file_path: String,
        error: String,
    },

    JobFinished { 
        name: String, 
        timestamp: String,
        failed_tracks: Vec<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CoreCommand {
    StartJob(String), // Job name
    StopJob(String),  // Job name
    GetStatus,        // Request current status of all jobs
    OpenUI,
    Shutdown
}