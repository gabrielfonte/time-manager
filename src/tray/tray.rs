use anyhow::Result;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread::{self, JoinHandle};

pub const OPEN_MENU_ID: &str = "open-time-manager";
pub const CLOSE_MENU_ID: &str = "close-time-manager";

pub struct TrayHandle {
    shutdown: Sender<()>,
    join_handle: JoinHandle<()>,
}

impl TrayHandle {
    pub fn shutdown(self) {
        let _ = self.shutdown.send(());
        let _ = self.join_handle.join();
    }
}

pub struct Tray;

impl Tray {
    pub fn init() -> Result<TrayHandle> {
        #[cfg(target_os = "linux")]
        {
            return linux::init();
        }

        #[cfg(any(target_os = "windows", target_os = "macos"))]
        {
            return desktop::init();
        }

        #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
        {
            Err(anyhow::anyhow!("Tray is not supported on this operating system"))
        }
    }
}

fn create_tray_icon() -> Result<tray_icon::TrayIcon> {
    let icon_bytes = include_bytes!("icon.ico");
    let image = image::load_from_memory(icon_bytes)?.into_rgba8();
    let (width, height) = image.dimensions();
    let icon = tray_icon::Icon::from_rgba(image.into_raw(), width, height)?;

    let open_item = tray_icon::menu::MenuItem::with_id(
        OPEN_MENU_ID,
        "Open Time Manager",
        true,
        None,
    );
    let close_item = tray_icon::menu::MenuItem::with_id(
        CLOSE_MENU_ID,
        "Close Time Manager",
        true,
        None,
    );
    let menu = tray_icon::menu::Menu::with_items(&[&open_item, &close_item])?;

    Ok(tray_icon::TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip("Time Manager")
        .with_icon(icon)
        .build()?)
}

#[cfg(target_os = "linux")]
mod linux {
    use super::*;

    pub(super) fn init() -> Result<TrayHandle> {
        let (shutdown_tx, shutdown_rx) = channel::<()>();
        let join_handle = thread::spawn(move || run(shutdown_rx));
        Ok(TrayHandle { shutdown: shutdown_tx, join_handle })
    }

    fn run(shutdown_rx: Receiver<()>) {
        if let Err(error) = gtk::init() {
            eprintln!("Failed to initialize GTK: {error:?}");
            return;
        }

        let tray_icon = match create_tray_icon() {
            Ok(icon) => icon,
            Err(error) => {
                eprintln!("Failed to create tray icon: {error}");
                return;
            }
        };

        let _tray_icon = tray_icon;
        thread::spawn(move || {
            let _ = shutdown_rx.recv();
            gtk::glib::MainContext::default().invoke(gtk::main_quit);
        });

        gtk::main();
    }
}

#[cfg(any(target_os = "windows", target_os = "macos"))]
mod desktop {
    use super::*;

    pub(super) fn init() -> Result<TrayHandle> {
        let (shutdown_tx, shutdown_rx) = channel::<()>();
        let join_handle = thread::spawn(move || run(shutdown_rx));
        Ok(TrayHandle { shutdown: shutdown_tx, join_handle })
    }

    fn run(shutdown_rx: Receiver<()>) {
        let tray_icon = match create_tray_icon() {
            Ok(icon) => icon,
            Err(error) => {
                eprintln!("Failed to create tray icon: {error}");
                return;
            }
        };

        let _tray_icon = tray_icon;
        let _ = shutdown_rx.recv();
    }
}
