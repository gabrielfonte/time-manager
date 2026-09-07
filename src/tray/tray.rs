use anyhow::Result;
use async_channel::{Receiver, Sender};
use std::sync::mpsc;
use std::thread::{self, JoinHandle};

pub const CLOSE_MENU_ID: &str = "close-time-manager";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayEvent {
    Close,
}

pub struct TrayHandle {
    shutdown: mpsc::Sender<()>,
    events: Receiver<TrayEvent>,
    join_handle: JoinHandle<()>,
}

impl TrayHandle {
    pub fn events(&self) -> Receiver<TrayEvent> {
        self.events.clone()
    }

    pub fn shutdown(self) {
        let _ = self.shutdown.send(());
        let _ = self.join_handle.join();
    }
}

pub struct Tray;

impl Tray {
    pub fn init() -> Result<TrayHandle> {
        let (event_tx, event_rx) = async_channel::unbounded::<TrayEvent>();

        #[cfg(target_os = "linux")]
        {
            return linux::init(event_tx, event_rx);
        }

        #[cfg(target_os = "windows")]
        {
            return windows::init(event_tx, event_rx);
        }

        #[cfg(target_os = "macos")]
        {
            return macos::init(event_tx, event_rx);
        }

        #[cfg(not(any(
            target_os = "linux",
            target_os = "windows",
            target_os = "macos"
        )))]
        {
            Err(anyhow::anyhow!(
                "Tray is not supported on this operating system"
            ))
        }
    }
}

fn create_tray_icon() -> Result<tray_icon::TrayIcon> {
    let icon_bytes = include_bytes!("icon.ico");
    let image = image::load_from_memory(icon_bytes)?.into_rgba8();
    let (width, height) = image.dimensions();

    let icon =
        tray_icon::Icon::from_rgba(image.into_raw(), width, height)?;

    let close_item = tray_icon::menu::MenuItem::with_id(
        CLOSE_MENU_ID,
        "close",
        true,
        None,
    );

    let menu = tray_icon::menu::Menu::with_items(&[&close_item])?;

    Ok(
        tray_icon::TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip("Time Manager")
            .with_icon(icon)
            .build()?,
    )
}

#[cfg(target_os = "linux")]
mod linux {
    use super::*;

    pub(super) fn init(
        event_tx: Sender<TrayEvent>,
        event_rx: Receiver<TrayEvent>,
    ) -> Result<TrayHandle> {
        let (shutdown_tx, shutdown_rx) = mpsc::channel::<()>();

        let join_handle =
            thread::spawn(move || run(shutdown_rx, event_tx));

        Ok(TrayHandle {
            shutdown: shutdown_tx,
            events: event_rx,
            join_handle,
        })
    }

    fn run(
        shutdown_rx: mpsc::Receiver<()>,
        event_tx: Sender<TrayEvent>,
    ) {
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

        tray_icon::menu::MenuEvent::set_event_handler(Some(
            move |event: tray_icon::menu::MenuEvent| {
                if event.id().as_ref() == CLOSE_MENU_ID {
                    let _ = event_tx.try_send(TrayEvent::Close);
                }
            },
        ));

        thread::spawn(move || {
            let _ = shutdown_rx.recv();

            gtk::glib::MainContext::default().invoke(|| {
                gtk::main_quit();
            });
        });

        gtk::main();

        tray_icon::menu::MenuEvent::set_event_handler(None::<fn(tray_icon::menu::MenuEvent)>);
    }
}

#[cfg(target_os = "windows")]
mod windows {
    use super::*;
    use std::mem::zeroed;
    use std::time::Duration;

    use windows_sys::Win32::UI::WindowsAndMessaging::{
        DispatchMessageW,
        PeekMessageW,
        TranslateMessage,
        MSG,
        PM_REMOVE,
    };

    pub(super) fn init(
        event_tx: Sender<TrayEvent>,
        event_rx: Receiver<TrayEvent>,
    ) -> Result<TrayHandle> {
        let (shutdown_tx, shutdown_rx) = mpsc::channel::<()>();

        let join_handle =
            thread::spawn(move || run(shutdown_rx, event_tx));

        Ok(TrayHandle {
            shutdown: shutdown_tx,
            events: event_rx,
            join_handle,
        })
    }

    fn run(
        shutdown_rx: mpsc::Receiver<()>,
        event_tx: Sender<TrayEvent>,
    ) {
        unsafe {
            let mut message: MSG = zeroed();

            PeekMessageW(
                &mut message,
                std::ptr::null_mut(),
                0,
                0,
                PM_REMOVE,
            );
        }

        let tray_icon = match create_tray_icon() {
            Ok(icon) => icon,
            Err(error) => {
                eprintln!("Failed to create tray icon: {error}");
                return;
            }
        };

        let _tray_icon = tray_icon;

        tray_icon::menu::MenuEvent::set_event_handler(Some(
            move |event: tray_icon::menu::MenuEvent| {
                if event.id().as_ref() == CLOSE_MENU_ID {
                    let _ = event_tx.try_send(TrayEvent::Close);
                }
            },
        ));

        loop {
            if shutdown_rx.try_recv().is_ok() {
                break;
            }

            unsafe {
                let mut message: MSG = zeroed();

                while PeekMessageW(
                    &mut message,
                    std::ptr::null_mut(),
                    0,
                    0,
                    PM_REMOVE,
                ) != 0
                {
                    TranslateMessage(&message);
                    DispatchMessageW(&message);
                }
            }

            thread::sleep(Duration::from_millis(10));
        }

        tray_icon::menu::MenuEvent::set_event_handler(None::<fn(tray_icon::menu::MenuEvent)>);
    }
}

#[cfg(target_os = "macos")]
mod macos {
    use super::*;

    pub(super) fn init(
        event_tx: Sender<TrayEvent>,
        event_rx: Receiver<TrayEvent>,
    ) -> Result<TrayHandle> {
        let (shutdown_tx, shutdown_rx) = mpsc::channel::<()>();

        let join_handle =
            thread::spawn(move || run(shutdown_rx, event_tx));

        Ok(TrayHandle {
            shutdown: shutdown_tx,
            events: event_rx,
            join_handle,
        })
    }

    fn run(
        shutdown_rx: mpsc::Receiver<()>,
        event_tx: Sender<TrayEvent>,
    ) {
        let tray_icon = match create_tray_icon() {
            Ok(icon) => icon,
            Err(error) => {
                eprintln!("Failed to create tray icon: {error}");
                return;
            }
        };

        let _tray_icon = tray_icon;

        tray_icon::menu::MenuEvent::set_event_handler(Some(
            move |event: tray_icon::menu::MenuEvent| {
                if event.id().as_ref() == CLOSE_MENU_ID {
                    let _ = event_tx.try_send(TrayEvent::Close);
                }
            },
        ));

        let _ = shutdown_rx.recv();

        tray_icon::menu::MenuEvent::set_event_handler(None::<fn(tray_icon::menu::MenuEvent)>);
    }
}