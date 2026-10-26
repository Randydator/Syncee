use super::list;
use crate::constants;
use crate::event::CoreCommand;
use iced::widget::{button, column, text};
use iced::{Element, Task};
use interprocess::local_socket::{GenericNamespaced, ToNsName, tokio::Stream, tokio::prelude::*};
use std::io;
use tokio::io::AsyncWriteExt;

pub fn run_gui() -> iced::Result {
    // Create a temporary tokio runtime context for the GUI process
    let tokio_runtime = tokio::runtime::Runtime::new().unwrap();
    let _guard = tokio_runtime.enter(); // This registers the Tokio reactor on this thread!

    iced::application(SynceeGui::default, SynceeGui::update, SynceeGui::view)
        .title("Syncee Control Panel")
        .run()
}

struct SynceeGui {
    // We hold a crossbeam or tokio mpsc channel to ship messages out to a background connection worker thread
    tx_to_daemon: Option<tokio::sync::mpsc::Sender<CoreCommand>>,
}

#[derive(Debug, Clone)]
pub enum GuiMessage {
    Connected(tokio::sync::mpsc::Sender<CoreCommand>),
    ConnectionFailed(String),
    SendOpenUI,
    SendShutdown,
}

// 1. Boot function: Returns the default starting state
// 2. Update function: Handles state modifications
// 3. View function: Builds the visual layout
impl SynceeGui {
    fn default() -> (Self, Task<GuiMessage>) {
        let initial_state = Self { tx_to_daemon: None };

        // Command Iced to run our async connector loop in the background immediately
        let connect_task = Task::perform(connect_to_daemon(), |result| match result {
            Ok(sender) => GuiMessage::Connected(sender),
            Err(err) => GuiMessage::ConnectionFailed(err.to_string()),
        });

        (initial_state, connect_task)
    }

    fn update(&mut self, message: GuiMessage) {
        match message {
            GuiMessage::Connected(sender) => {
                log::info!("GUI: Secure IPC pipe connection established.");
                self.tx_to_daemon = Some(sender);
            }
            GuiMessage::ConnectionFailed(err) => {
                log::error!("GUI: Connection failed: {}", err);
            }
            GuiMessage::SendOpenUI => {
                if let Some(tx) = &self.tx_to_daemon {
                    let _ = tx.try_send(CoreCommand::OpenUI);
                }
            }
            GuiMessage::SendShutdown => {
                if let Some(tx) = &self.tx_to_daemon {
                    let _ = tx.try_send(CoreCommand::Shutdown);
                }
            }
        }
    }

    fn view(&self) -> Element<GuiMessage> {
        column![
            text("Syncee Control Panel").size(28),
            text(if self.tx_to_daemon.is_some() {
                "Status: Linked via IPC Pipe"
            } else {
                "Status: Disconnected"
            }),
            button("Send OpenUI Command").on_press(GuiMessage::SendOpenUI),
            button("Kill Background Daemon").on_press(GuiMessage::SendShutdown),
        ]
        .spacing(15)
        .align_x(iced::Alignment::Center)
        .into() // CRITICAL: Add .into() at the end to convert the Column to an Element
    }
}

// ICED update is a sync process. Task::perform is the way to make async tasks.
// They will run in the background, then come as a message to GUI in another update loop.
// Because I want to connect once to the backend and then have a persistent connection (not connect per message)
// We here need to spawn a new tokio task to always run in the background with the GUI, which handles backend communication without reconnect
async fn connect_to_daemon() -> Result<tokio::sync::mpsc::Sender<CoreCommand>, std::io::Error> {
    // Try to connect to tokio runtime IPC pipe using name for backend communication. Namespaced needed to handle cross-plattform socket names.
    let name = constants::PIPE_NAME
        .to_ns_name::<GenericNamespaced>()
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;

    let stream = Stream::connect(name)
        .await
        .inspect_err(|e| log::error!("GUI connection to backend failed: {e}"))?;

    //Currently unused reader, as Backend to UI not implemented yet
    let (mut reader, mut writer) = stream.split();

    // Create always running tokio channel that is connected to backend for communication
    let (tx, mut rx) = tokio::sync::mpsc::channel::<CoreCommand>(32);
    tokio::spawn(async move {
        while let Some(command) = rx.recv().await {
            // Serialize your type-safe Enum into raw binary bytes
            if let Ok(payload) = bincode::serialize(&command) {
                if let Err(e) = writer.write_all(&payload).await {
                    log::error!("IPC Worker failed to write payload to pipe: {}", e);
                    break;
                }
                let _ = writer.flush().await;
            }
        }
    });

    // Return transmitter so UI can use this to send messages to backend
    Ok(tx)
}
