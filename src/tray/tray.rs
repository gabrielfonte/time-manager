use anyhow::Result;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread::{self, JoinHandle};

pub const CLOSE_MENU_ID: &str = "close-time-manager";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayEvent {
    Close,
}

pub struct TrayHandle {
    shutdown: Sender<()>,
    events: Option<Receiver<TrayEvent>>,
    join_handle: JoinHandle<()>,
}

impl TrayHandle {
    pub fn shutdown(self) {
        let _ = self.shutdown.send(());
        let _ = self.join_handle.join();
    }

    pub fn take_events(&mut self) -> Receiver<TrayEvent> {
        self.events.take().expect("tray events already taken")
    }
}

pub struct Tray;

impl Tray {
    pub fn init() -> Result<TrayHandle> {
        let (event_tx, event_rx) = channel::<TrayEvent>();

        #[cfg(target_os = "linux")]
        {
            return linux::init(event_tx, event_rx);
        }

        #[cfg(any(target_os = "windows", target_os = "macos"))]
        {
            return desktop::init(event_tx, event_rx);
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

    let close_item = tray_icon::menu::MenuItem::with_id(
        CLOSE_MENU_ID,
        "close",
        true,
        None,
    );
    let menu = tray_icon::menu::Menu::with_items(&[&close_item])?;

    Ok(tray_icon::TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip("Time Manager")
        .with_icon(icon)
        .build()?)
}

#[cfg(target_os = "linux")]
mod linux {
    use super::*;

    pub(super) fn init(event_tx: Sender<TrayEvent>, event_rx: Receiver<TrayEvent>) -> Result<TrayHandle> {
        let (shutdown_tx, shutdown_rx) = channel::<()>();
        let join_handle = thread::spawn(move || run(shutdown_rx, event_tx));
        Ok(TrayHandle { shutdown: shutdown_tx, events: Some(event_rx), join_handle })
    }

    fn run(shutdown_rx: Receiver<()>, event_tx: Sender<TrayEvent>) {
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
        tray_icon::menu::MenuEvent::set_event_handler(Some(move |event: tray_icon::menu::MenuEvent| {
            let tray_event = match event.id().as_ref() {
                CLOSE_MENU_ID => TrayEvent::Close,
                _ => return,
            };
            let _ = event_tx.send(tray_event);
        }));
        thread::spawn(move || {
            let _ = shutdown_rx.recv();
            gtk::glib::MainContext::default().invoke(gtk::main_quit);
        });

        gtk::main();
    }
}

#[cfg(target_os = "windows")]
mod desktop {
    use super::*;
    use winit::application::ApplicationHandler;
    use winit::event::WindowEvent;
    use winit::event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy};
    use winit::window::WindowId;

    enum UserEvent {
        Shutdown,
    }

    struct TrayApplication {
        event_tx: Sender<TrayEvent>,
        tray_icon: Option<tray_icon::TrayIcon>,
    }

    impl ApplicationHandler<UserEvent> for TrayApplication {
        fn resumed(&mut self, _event_loop: &ActiveEventLoop) {
            if self.tray_icon.is_some() {
                return;
            }

            let tray_icon = match create_tray_icon() {
                Ok(icon) => icon,
                Err(error) => {
                    eprintln!("Failed to create tray icon: {error}");
                    return;
                }
            };

            let event_tx = self.event_tx.clone();
            tray_icon::menu::MenuEvent::set_event_handler(Some(move |event: tray_icon::menu::MenuEvent| {
                if event.id().as_ref() == CLOSE_MENU_ID {
                    let _ = event_tx.send(TrayEvent::Close);
                }
            }));
            self.tray_icon = Some(tray_icon);
        }

        fn user_event(&mut self, event_loop: &ActiveEventLoop, event: UserEvent) {
            match event {
                UserEvent::Shutdown => event_loop.exit(),
            }
        }

        fn window_event(
            &mut self,
            _event_loop: &ActiveEventLoop,
            _window_id: WindowId,
            _event: WindowEvent,
        ) {
        }
    }

    pub(super) fn init(event_tx: Sender<TrayEvent>, event_rx: Receiver<TrayEvent>) -> Result<TrayHandle> {
        let (shutdown_tx, shutdown_rx) = channel::<()>();
        let join_handle = thread::spawn(move || run(shutdown_rx, event_tx));
        Ok(TrayHandle { shutdown: shutdown_tx, events: Some(event_rx), join_handle })
    }

    fn run(shutdown_rx: Receiver<()>, event_tx: Sender<TrayEvent>) {
        let event_loop = match EventLoop::<UserEvent>::with_user_event().build() {
            Ok(event_loop) => event_loop,
            Err(error) => {
                eprintln!("Failed to create tray event loop: {error}");
                return;
            }
        };
        let proxy: EventLoopProxy<UserEvent> = event_loop.create_proxy();
        thread::spawn(move || {
            let _ = shutdown_rx.recv();
            let _ = proxy.send_event(UserEvent::Shutdown);
        });

        let mut application = TrayApplication {
            event_tx,
            tray_icon: None,
        };
        if let Err(error) = event_loop.run_app(&mut application) {
            eprintln!("Tray event loop failed: {error}");
        }
    }
}

#[cfg(target_os = "macos")]
mod desktop {
    use super::*;

    pub(super) fn init(event_tx: Sender<TrayEvent>, event_rx: Receiver<TrayEvent>) -> Result<TrayHandle> {
        let (shutdown_tx, shutdown_rx) = channel::<()>();
        let join_handle = thread::spawn(move || run(shutdown_rx, event_tx));
        Ok(TrayHandle { shutdown: shutdown_tx, events: Some(event_rx), join_handle })
    }

    fn run(shutdown_rx: Receiver<()>, event_tx: Sender<TrayEvent>) {
        let tray_icon = match create_tray_icon() {
            Ok(icon) => icon,
            Err(error) => {
                eprintln!("Failed to create tray icon: {error}");
                return;
            }
        };

        let _tray_icon = tray_icon;
        tray_icon::menu::MenuEvent::set_event_handler(Some(move |event: tray_icon::menu::MenuEvent| {
            if event.id().as_ref() == CLOSE_MENU_ID {
                let _ = event_tx.send(TrayEvent::Close);
            }
        }));
        let _ = shutdown_rx.recv();
    }
}
