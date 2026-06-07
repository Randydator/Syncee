mod tray;

use core_engine::CoreBackend;
use core_engine::event::{CoreCommand, CoreEvent};
use iced::futures::StreamExt;
use iced::widget::{center, text};
use iced::{Subscription, Task};
use tokio_stream::wrappers::ReceiverStream;
use tray_icon::TrayIconEvent;
use tray_icon::menu::MenuEvent; // Required to use .map() on stream

fn open_window() -> Task<Message> {
    let (_id, open_task) = iced::window::open(iced::window::Settings {
        size: iced::Size::new(600.0, 400.0),
        ..Default::default()
    });
    
    open_task.discard() // destroy open task since we don't need to use the returning window id for anything
}

fn tray_native_stream() -> impl iced::futures::Stream<Item = Message> {
    // This gives us an 'UnboundedSender', which doesn't require mutable access to send data!
    let (transmitter, receiver) = iced::futures::channel::mpsc::unbounded::<Message>();

    let tray_transmitter = transmitter.clone();
    let menu_transmitter = transmitter;

    // move so that the transmitter don't get destroyed when this function ends, and .unbounded_send() works on a plain &tx, keeping the closures compliant
    TrayIconEvent::set_event_handler(Some(move |event| {
        // .unbounded_send() works on a plain &tx, keeping the closure compliant!
        let _ = tray_transmitter.unbounded_send(Message::TrayInteraction(event));
    }));

    MenuEvent::set_event_handler(Some(move |event| {
        // .unbounded_send() works on a plain &tx, keeping the closure compliant!
        let _ = menu_transmitter.unbounded_send(Message::MenuInteraction(event));
    }));

    // 3. Return the receiver straight to Iced
    receiver
}

pub fn main() -> iced::Result {
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
        // setup communication channels; mpsc = multi-producer, single-consumer
        let (to_core, from_ui) = tokio::sync::mpsc::channel::<CoreCommand>(32);
        let (to_ui, from_core) = tokio::sync::mpsc::channel::<CoreEvent>(32);

        let engine = CoreBackend::new(from_ui, to_ui);
        tokio::spawn(engine.run());

        let tray_assets = tray::setup_tray_menu();

        let backend_stream = ReceiverStream::new(from_core).map(Message::BackendEvent);

        let listen_to_backend = Task::run(
            backend_stream,
            |message| message, // passes message straight to update()
        );

        (
            Self {
                to_core,
                tray_assets,
                last_backend_log: String::from("Waiting for backend events..."),
            },
            listen_to_backend,
        )
    }

    fn subscription(&self) -> Subscription<Message> {
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
                    log::info!(
                        "Iced Loop: 'Show UI' menu item clicked. Opening winow"
                    );
                    return open_window();
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

    // 'a is the lifetime of the returned Element, which is tied to the lifetime of self. element doesn't have static time to live, it is linked to self
    fn view<'a>(&'a self, _window_id: iced::window::Id) -> iced::Element<'a, Message> {
        center(text(format!("Syncee Utility Running\n{}", self.last_backend_log)).size(20)).into()
    }
}
