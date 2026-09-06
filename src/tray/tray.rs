use anyhow::Result;
use std::sync::mpsc::{channel, Sender};
use std::thread::{self, JoinHandle};

pub struct TrayHandle {
    shutdown: Sender<()>,
    join_handle: JoinHandle<()>,
}

impl TrayHandle {
    // Request the tray thread to shutdown and wait for it to exit.
    pub fn shutdown(self) {
        let _ = self.shutdown.send(());
        let _ = self.join_handle.join();
    }
}

pub struct GtkTray;

impl GtkTray {
    pub fn init() -> Result<TrayHandle> {
        let (tx, rx) = channel::<()>();

        let join_handle = thread::spawn(move || {
            // Initialize GTK on this thread and create the tray icon here.
            if let Err(e) = gtk::init() {
                eprintln!("Failed to initialize GTK: {:?}", e);
                return;
            }

            // Load icon bytes (path relative to this file: src/tray/icon.ico)
            let icon_bytes = include_bytes!("icon.ico");
            let image = match image::load_from_memory(icon_bytes) {
                Ok(img) => img.into_rgba8(),
                Err(err) => {
                    eprintln!("Failed to load icon: {}", err);
                    return;
                }
            };

            let (width, height) = image.dimensions();
            let rgba_raw = image.into_raw();

            let icon = match tray_icon::Icon::from_rgba(rgba_raw, width, height) {
                Ok(i) => i,
                Err(err) => {
                    eprintln!("Failed to create tray icon: {}", err);
                    return;
                }
            };

            // Build a simple empty menu (adding items is backend-dependent)
            let tray_menu = tray_icon::menu::Menu::new();

            let tray_icon = match tray_icon::TrayIconBuilder::new()
                .with_menu(Box::new(tray_menu))
                .with_tooltip("Time Manager")
                .with_icon(icon)
                .build()
            {
                Ok(t) => t,
                Err(err) => {
                    eprintln!("Failed to build tray icon: {}", err);
                    return;
                }
            };

            // Listen for a shutdown request on a background thread so we can call
            // gtk::main_quit() from this GTK thread when requested.
            let shutdown_rx = rx;
            let _tray_icon = tray_icon; // keep alive

            // Spawn a helper thread to wait for shutdown and quit the GTK main loop.
            let _waiter = std::thread::spawn(move || {
                // Block until sender sends
                let _ = shutdown_rx.recv();
                // Request GTK main loop to quit
                gtk::main_quit();
            });

            // Run GTK main loop on this thread (blocks until gtk::main_quit())
            gtk::main();
        });

        Ok(TrayHandle { shutdown: tx, join_handle })
    }
}