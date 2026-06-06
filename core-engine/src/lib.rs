pub mod event;

use event::{CoreCommand, CoreEvent};
use tokio::sync::mpsc::{Receiver, Sender};

pub struct CoreBackend {
    from_ui: Receiver<CoreCommand>,
    to_ui: Sender<CoreEvent>,
}

impl CoreBackend {
    pub fn new(from_ui: Receiver<CoreCommand>, to_ui: Sender<CoreEvent>) -> Self {
        Self { from_ui, to_ui }
    }

    pub async fn run(mut self) {
        log::info!("Background sync engine worker is up and running...");

        while let Some(cmd) = self.from_ui.recv().await {
            match cmd {
                CoreCommand::OpenUI => {
                    log::info!("Core Engine: Received OpenUi. Preparing background state...");
                }
                CoreCommand::Shutdown => {
                    log::info!("Core Engine: Shutting down background processes cleanly...");
                    // loop breaks > run method exits > fromCore.recv() in main.rs gets None > main loop breaks > app exits gracefully
                    break;
                }
                _ => {} // Handle sync jobs here later
            }
        }
    }
}