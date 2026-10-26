#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // No terminal for windows release builds

mod tray;
use crossbeam_channel::select;
use syncee::CoreBackend;
use syncee::ui::gui;
use tray_icon::{TrayIconEvent, menu::MenuEvent};

fn handle_gui_mode_if_set() -> bool {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "--mode=gui" {
        log::info!("Syncee: GUI flag detected. Booting GUI...");

        if let Err(e) = gui::run_gui() {
            log::error!("GUI process context exited with error: {}", e);
        }

        return true;
    }
    false // Not in GUI mode
}

struct ThreadSafeTrayIds {
    show_ui_id: tray_icon::menu::MenuId,
    quit_id: tray_icon::menu::MenuId,
}

fn spawn_tray_loop() {
    let tray_assets = tray::setup_tray_menu();
    let tray_receiver = TrayIconEvent::receiver().clone();
    let menu_receiver = MenuEvent::receiver().clone();

    // Extract the thread-safe IDs out of the assets structure
    let tray_ids = ThreadSafeTrayIds {
        show_ui_id: tray_assets.show_ui_id.clone(),
        quit_id: tray_assets.quit_id.clone(),
    };

    // Spawn the background thread and pass ONLY the thread-safe IDs
    std::thread::spawn(move || {
        handle_tray_events(tray_receiver, menu_receiver, tray_ids);
    });

    log::info!("Tray loop initialized. Main thread entering sleep/message state.");

    // CRITICAL: We leak or anchor tray_assets here so it stays alive for the duration of main()
    // If it gets dropped here, your icon disappears from the taskbar instantly!
    std::mem::forget(tray_assets);
}

fn handle_tray_events(
    tray_receiver: crossbeam_channel::Receiver<TrayIconEvent>,
    menu_receiver: crossbeam_channel::Receiver<MenuEvent>,
    tray_ids: ThreadSafeTrayIds,
) {
    loop {
        select! {
            recv(tray_receiver) -> tray_message => {
                if let Ok(event) = tray_message {
                    if let TrayIconEvent::Click { button, button_state, .. } = event {
                        if button == tray_icon::MouseButton::Left && button_state == tray_icon::MouseButtonState::Up {
                            log::info!("Daemon: Tray left-clicked. Spawning GUI binary!");
                            spawn_gui_process();
                        }
                    }
                }
            }

            recv(menu_receiver) -> menu_message => {
                if let Ok(event) = menu_message {
                    // FIXED: Match against tray_ids instead of tray_assets
                    if event.id == tray_ids.show_ui_id {
                        log::info!("Daemon: 'Open Configuration' clicked. Spawning GUI binary!");
                        spawn_gui_process();
                    } else if event.id == tray_ids.quit_id {
                        log::info!("Daemon: 'Quit Application' clicked. Shutting down service cleanly.");
                        std::process::exit(0);
                    }
                }
            }
        }
    }
}

fn spawn_gui_process() {
    // Look up the exact path of this executable running right now on the filesystem
    if let Ok(current_exe) = std::env::current_exe() {
        // Run a twin copy of ourselves, but pass the secret flag!
        if let Err(e) = std::process::Command::new(current_exe)
            .arg("--mode=gui")
            .spawn()
        {
            log::error!("Daemon failed to self-spawn GUI process instance: {}", e);
        }
    }
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    log::info!("Syncee started...");

    if handle_gui_mode_if_set() {
        return;
    }

    log::info!("Syncee Daemon booting up...");
    let runtime = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");

    runtime.spawn(async move {
        let engine = CoreBackend::new();
        if let Err(e) = engine.run().await {
            log::error!("Core Engine background runtime dropped with error: {}", e);
        }
    });

    spawn_tray_loop();

    #[cfg(target_os = "windows")]
    unsafe {
        let mut msg = std::mem::zeroed();

        // Use std::ptr::null_mut() instead of 0 for the HWND argument
        while windows_sys::Win32::UI::WindowsAndMessaging::GetMessageW(
            &mut msg,
            std::ptr::null_mut(), // Fixes the expected *mut c_void error
            0,
            0,
        ) > 0
        {
            windows_sys::Win32::UI::WindowsAndMessaging::TranslateMessage(&msg);
            windows_sys::Win32::UI::WindowsAndMessaging::DispatchMessageW(&msg);
        }
    }
}
