pub mod constants;
pub mod event;
pub mod ui;

use event::CoreCommand;
use interprocess::local_socket::{
    GenericNamespaced,
    ListenerOptions,
    // CHANGE THIS: Target the .tokio.prelude module specifically
    tokio::prelude::*,
};
use std::io;
use tokio::io::AsyncReadExt;

pub struct CoreBackend {}

impl CoreBackend {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn run(self) -> io::Result<()> {
        log::info!("Background sync engine worker is up and running...");

        // Namespaced for cross-plattform socket name handling
        let name = constants::PIPE_NAME
            .to_ns_name::<GenericNamespaced>()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;

        let listener = ListenerOptions::new().name(name).create_tokio()?;
        log::info!("IPC Listener bound to name: {}", constants::PIPE_NAME);

        loop {
            // awaits for anything to connect to the socket
            match listener.accept().await {
                Ok(stream) => {
                    log::info!("GUI client connected! Starting IPC handler loop...");

                    // Use tokio::spawn so a single client context can run concurrently,
                    // keeping the main loop ready to accept immediate future handshakes.
                    tokio::spawn(async move {
                        if let Err(e) = Self::handle_client(stream).await {
                            log::error!("Error handling IPC client stream: {}", e);
                        }
                        log::info!("GUI client disconnected. Returning to listen state.");
                    });
                }
                Err(e) => log::error!("Failed to accept incoming IPC connection: {}", e),
            }
        }
    }

    async fn handle_client(stream: interprocess::local_socket::tokio::Stream) -> io::Result<()> {
        let (mut reader, _writer) = stream.split();
        // block 1024 bytes of memory (initialized as 0s)
        let mut buffer = vec![0u8; 1024];

        loop {
            let bytes_read = reader.read(&mut buffer).await?;
            if bytes_read == 0 {
                break; // Stream reached EOF (GUI closed)
            }

            // Deserialize the raw bytes back into your clean Rust enum
            if let Ok(cmd) = bincode::deserialize::<CoreCommand>(&buffer[..bytes_read]) {
                match cmd {
                    CoreCommand::OpenUI => {
                        log::info!("Received OpenUI");
                    }
                    CoreCommand::Shutdown => {
                        log::info!("Shutting down Backend. Bye byeee");
                        std::process::exit(0);
                    }
                    _ => {
                        log::info!("Unknown command received: {:?}", cmd);
                    }
                }
            }
        }
        Ok(())
    }
}
