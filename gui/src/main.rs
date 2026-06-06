mod tray;

use core_engine::CoreBackend;
use core_engine::event::{CoreCommand, CoreEvent};
use iced::Event;
use iced::widget::{center, text};
use iced::{Subscription, Task};
use tray_icon::TrayIconEvent;
use tray_icon::menu::MenuEvent;

fn tray_native_stream() -> impl iced::futures::Stream<Item = Message> {
    // 1. Use an unbounded channel.
    // This gives us an 'UnboundedSender', which doesn't require mutable access to send data!
    let (tx, rx) = iced::futures::channel::mpsc::unbounded::<Message>();

    let tray_tx = tx.clone();
    let menu_tx = tx;

    // 2. Pass the handler. Because tx can be used immutably, this remains a pure `Fn` closure.
    TrayIconEvent::set_event_handler(Some(move |event| {
        // .unbounded_send() works on a plain &tx, keeping the closure compliant!
        let _ = tray_tx.unbounded_send(Message::TrayInteraction(event));
    }));

    MenuEvent::set_event_handler(Some(move |event| {
        // .unbounded_send() works on a plain &tx, keeping the closure compliant!
        let _ = menu_tx.unbounded_send(Message::MenuInteraction(event));
    }));

    // 3. Return the receiver straight to Iced
    rx
}

#[tokio::main]
pub async fn main() -> iced::Result {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    log::info!("Starting Syncee App with Iced Framework...");

    iced::daemon(SynceeApp::new, SynceeApp::update, SynceeApp::view)
        .title("Syncee")
        .subscription(SynceeApp::subscription)
        .run() // <-- Merged run execution
}

struct SynceeApp {
    to_core: tokio::sync::mpsc::Sender<CoreCommand>,
    tray_assets: tray::SystemTrayAssets,
    last_backend_log: String,
}

#[derive(Debug, Clone)]
enum Message {
    // Routed from our Core Engine Backend
    BackendEvent(CoreEvent),
    // Routed from the taskbar interactions
    TrayInteraction(tray_icon::TrayIconEvent),
    MenuInteraction(tray_icon::menu::MenuEvent),
}

impl SynceeApp {
    fn new() -> (Self, Task<Message>) {
        // 1. Set up the workspace messaging channels
        let (to_core, from_ui) = tokio::sync::mpsc::channel::<CoreCommand>(32);
        let (to_ui, from_core) = tokio::sync::mpsc::channel::<CoreEvent>(32);

        // 2. Spin up the backend worker task using Iced's shared Tokio runtime context
        let engine = CoreBackend::new(from_ui, to_ui);
        tokio::spawn(engine.run());

        // 3. Render the native system tray icon
        let tray_assets = tray::setup_tray_menu();

        (
            Self {
                to_core,
                tray_assets,
                last_backend_log: String::from("Waiting for backend events..."),
            },
            Task::none(),
        )
    }

    fn subscription(&self) -> Subscription<Message> {
        // This is all you need! It passes your clean, event-driven stream constructor
        // straight to Iced without any mismatched closure arguments.
        Subscription::run(tray_native_stream)
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::TrayInteraction(event) => {
                //log::info!("This triggers on every tray interaction");
                if let TrayIconEvent::Click {
                    button,
                    button_state,
                    ..
                } = event
                {
                    if button == tray_icon::MouseButton::Left
                        && button_state == tray_icon::MouseButtonState::Up
                    {
                        log::info!("Iced Loop: Tray icon left-clicked shortcut triggered.");
                        let _ = self.to_core.try_send(CoreCommand::OpenUI);
                    }
                }
            }
            Message::MenuInteraction(event) => {
                if event.id == self.tray_assets.show_ui_id {
                    log::info!("Iced Loop: 'Open Configuration' menu item clicked.");
                    let _ = self.to_core.try_send(CoreCommand::OpenUI);
                } else if event.id == self.tray_assets.quit_id {
                    log::info!(
                        "Iced Loop: 'Quit Application' menu item clicked. Sending shutdown command to core."
                    );
                    let _ = self.to_core.try_send(CoreCommand::Shutdown);
                }
            }
            Message::BackendEvent(event) => match event {
                CoreEvent::JobStarted { name, .. } => {
                    self.last_backend_log = format!("Job started: {}", name);
                }
                _ => {}
            },
        }
        Task::none()
    }

    fn view(&self, _window_id: iced::window::Id) -> iced::Element<Message> {
        // A minimal text display layout inside your window
        center(text(format!("Syncee Utility Running\n{}", self.last_backend_log)).size(20)).into()
    }
}
