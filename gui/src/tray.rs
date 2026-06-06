use tray_icon::{
    Icon, MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent,
    menu::{Menu, MenuEvent, MenuItem, MenuItemBuilder},
};

pub struct SystemTrayAssets {
    pub tray_icon: TrayIcon,
    pub show_ui_id: tray_icon::menu::MenuId,
    pub quit_id: tray_icon::menu::MenuId,
}

fn load_icon() -> Icon {
    let icon_bytes = include_bytes!("../assets/syncee_icon.ico");

    let image_buffer = image::load_from_memory_with_format(icon_bytes, image::ImageFormat::Ico)
        .expect("Failed to decode embedded .ico file")
        .into_rgba8(); // format Rust expects

    let (width, height) = image_buffer.dimensions();
    let rgba_raw_pixels = image_buffer.into_raw();

    Icon::from_rgba(rgba_raw_pixels, width, height)
        .expect("Failed to generate tray-icon compatible structure")
}

pub fn setup_tray_menu() -> SystemTrayAssets {
    let tray_menu = Menu::new();

    let show_ui_item = MenuItemBuilder::new()
        .text("Open Configuration")
        .enabled(true)
        .build();

    let quit_item = MenuItemBuilder::new()
        .text("Quit Application")
        .enabled(true)
        .build();

    tray_menu
        .append_items(&[&show_ui_item, &quit_item])
        .expect("Failed to build system tray menu");

    let tray_icon = TrayIconBuilder::new()
        .with_tooltip("Syncee - Music Sync Utility")
        .with_icon(load_icon())
        .with_menu(Box::new(tray_menu))
        .build()
        .expect("Failed to create system tray icon");

    let show_ui_id = show_ui_item.id().clone();
    let quit_id = quit_item.id().clone();

    SystemTrayAssets {
        tray_icon,
        show_ui_id,
        quit_id,
    }
}