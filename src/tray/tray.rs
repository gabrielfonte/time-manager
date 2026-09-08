use anyhow::{Context, Result};
use async_channel::{Receiver, Sender};
use std::sync::{Mutex, Once, mpsc};
use std::thread::{self, JoinHandle};

pub const CLOSE_MENU_ID: &str = "close-time-manager";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayEvent {
    Close,
}

/// Owns the tray worker. Explicit shutdown or Drop signals and joins it.
pub struct TrayHandle {
    shutdown: Option<Box<dyn FnOnce() + Send>>,
    events: Receiver<TrayEvent>,
    join_handle: Option<JoinHandle<()>>,
}

impl TrayHandle {
    pub fn events(&self) -> Receiver<TrayEvent> {
        self.events.clone()
    }

    pub fn shutdown(mut self) {
        self.stop();
    }

    fn stop(&mut self) {
        if let Some(shutdown) = self.shutdown.take() {
            shutdown();
        }
        if let Some(worker) = self.join_handle.take() {
            if worker.join().is_err() {
                eprintln!("Tray worker panicked during shutdown");
            }
        }
    }
}

impl Drop for TrayHandle {
    fn drop(&mut self) {
        self.stop();
    }
}

// Do not return a handle before native initialization has succeeded.
// Dropping the bootstrap sender also reports a worker panic to the caller.
fn start_worker(
    events: Receiver<TrayEvent>,
    shutdown: impl FnOnce() + Send + 'static,
    run: impl FnOnce(mpsc::SyncSender<Result<()>>) + Send + 'static,
) -> Result<TrayHandle> {
    let (ready_tx, ready_rx) = mpsc::sync_channel(1);
    let worker = thread::Builder::new()
        .name("time-manager-tray".into())
        .spawn(move || run(ready_tx))
        .context("Failed to spawn tray worker")?;
    let handle = TrayHandle {
        shutdown: Some(Box::new(shutdown)),
        events,
        join_handle: Some(worker),
    };
    ready_rx
        .recv()
        .context("Tray worker exited before initialization")??;
    Ok(handle)
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
        #[cfg(not(any(target_os = "linux", target_os = "windows")))]
        {
            let _ = (event_tx, event_rx);
            anyhow::bail!("Tray requires Windows or Linux; macOS needs main-thread integration")
        }
    }
}

// muda 0.19 installs its global handler only once. Keep that handler stable,
// but release its channel after the native tray has been destroyed.
static MENU_TARGET: Mutex<Option<Sender<TrayEvent>>> = Mutex::new(None);
static MENU_HANDLER: Once = Once::new();

fn forward_menu(id: &str, sender: &Sender<TrayEvent>) {
    if id == CLOSE_MENU_ID {
        let _ = sender.try_send(TrayEvent::Close);
    }
}

struct MenuRoute;

impl MenuRoute {
    fn install(sender: Sender<TrayEvent>) -> Result<Self> {
        let mut target = MENU_TARGET.lock().unwrap_or_else(|e| e.into_inner());
        anyhow::ensure!(target.is_none(), "A tray is already active");
        *target = Some(sender);
        MENU_HANDLER.call_once(|| {
            tray_icon::menu::MenuEvent::set_event_handler(Some(
                |event: tray_icon::menu::MenuEvent| {
                    let target = MENU_TARGET.lock().unwrap_or_else(|e| e.into_inner());
                    if let Some(sender) = target.as_ref() {
                        forward_menu(event.id().as_ref(), sender);
                    }
                },
            ));
        });
        Ok(Self)
    }
}

impl Drop for MenuRoute {
    fn drop(&mut self) {
        MENU_TARGET.lock().unwrap_or_else(|e| e.into_inner()).take();
    }
}

fn create_tray_icon() -> Result<tray_icon::TrayIcon> {
    let image = image::load_from_memory(include_bytes!("icon.ico"))?.into_rgba8();
    let (width, height) = image.dimensions();
    let icon = tray_icon::Icon::from_rgba(image.into_raw(), width, height)?;
    let quit = tray_icon::menu::MenuItem::with_id(CLOSE_MENU_ID, "Quit", true, None);
    let menu = tray_icon::menu::Menu::with_items(&[&quit])?;
    Ok(tray_icon::TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip("Time Manager")
        .with_icon(icon)
        .build()?)
}

#[cfg(target_os = "linux")]
mod linux {
    use super::*;
    use gtk::glib::{ControlFlow, MainContext, Priority};
    use std::sync::OnceLock;

    // GTK cannot be reinitialized on another worker thread after shutdown.
    static GTK_STARTED: OnceLock<()> = OnceLock::new();

    #[allow(deprecated)] // Required GLib channel API, supported by pinned gtk/glib 0.18.
    pub(super) fn init(
        event_tx: Sender<TrayEvent>,
        event_rx: Receiver<TrayEvent>,
    ) -> Result<TrayHandle> {
        anyhow::ensure!(
            GTK_STARTED.set(()).is_ok(),
            "GTK tray can only be initialized once per process"
        );
        let (shutdown_tx, shutdown_rx) = MainContext::channel::<()>(Priority::DEFAULT);
        start_worker(
            event_rx,
            move || {
                let _ = shutdown_tx.send(());
            },
            move |ready| {
                let initialized = (|| -> Result<_> {
                    gtk::init()
                        .context("Failed to initialize GTK (check DISPLAY/Wayland session)")?;
                    let route = MenuRoute::install(event_tx)?;
                    let icon = create_tray_icon().context("Failed to create GTK tray icon")?;
                    Ok((route, icon))
                })();
                let (route, icon) = match initialized {
                    Ok(resources) => resources,
                    Err(error) => {
                        let _ = ready.send(Err(error));
                        return;
                    }
                };
                let context = MainContext::default();
                let source = shutdown_rx.attach(Some(&context), |_| {
                    gtk::main_quit();
                    ControlFlow::Break
                });
                if ready.send(Ok(())).is_err() {
                    source.remove();
                    return;
                }
                // An early shutdown stays queued until this loop starts.
                gtk::main();
                drop(icon);
                drop(route);
            },
        )
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        #[allow(deprecated)]
        fn shutdown_queued_before_loop_is_delivered() {
            let context = MainContext::new();
            let _owner = context.acquire().unwrap();
            let (tx, rx) = MainContext::channel::<()>(Priority::DEFAULT);
            let called = std::rc::Rc::new(std::cell::Cell::new(false));
            let result = called.clone();
            rx.attach(Some(&context), move |_| {
                result.set(true);
                ControlFlow::Break
            });
            tx.send(()).unwrap();
            context.iteration(false);
            assert!(called.get());
        }
    }
}

#[cfg(target_os = "windows")]
mod windows {
    use super::*;
    use std::time::Duration;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        DispatchMessageW, MSG, PM_REMOVE, PeekMessageW, TranslateMessage,
    };

    pub(super) fn init(
        event_tx: Sender<TrayEvent>,
        event_rx: Receiver<TrayEvent>,
    ) -> Result<TrayHandle> {
        let (shutdown_tx, shutdown_rx) = mpsc::channel();
        start_worker(
            event_rx,
            move || {
                let _ = shutdown_tx.send(());
            },
            move |ready| {
                let initialized = (|| -> Result<_> {
                    unsafe {
                        let mut message: MSG = std::mem::zeroed();
                        PeekMessageW(&mut message, std::ptr::null_mut(), 0, 0, PM_REMOVE);
                    }
                    let route = MenuRoute::install(event_tx)?;
                    let icon = create_tray_icon().context("Failed to create Windows tray icon")?;
                    Ok((route, icon))
                })();
                let (route, icon) = match initialized {
                    Ok(resources) => resources,
                    Err(error) => {
                        let _ = ready.send(Err(error));
                        return;
                    }
                };
                if ready.send(Ok(())).is_err() {
                    return;
                }
                loop {
                    match shutdown_rx.try_recv() {
                        Ok(()) | Err(mpsc::TryRecvError::Disconnected) => break,
                        Err(mpsc::TryRecvError::Empty) => {}
                    }
                    unsafe {
                        let mut message: MSG = std::mem::zeroed();
                        while PeekMessageW(&mut message, std::ptr::null_mut(), 0, 0, PM_REMOVE) != 0
                        {
                            TranslateMessage(&message);
                            DispatchMessageW(&message);
                        }
                    }
                    thread::sleep(Duration::from_millis(10));
                }
                drop(icon);
                drop(route);
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};

    #[test]
    fn quit_is_forwarded_and_unknown_menu_is_ignored() {
        let (tx, rx) = async_channel::unbounded();
        forward_menu("unknown", &tx);
        assert!(rx.try_recv().is_err());
        forward_menu(CLOSE_MENU_ID, &tx);
        assert_eq!(rx.try_recv().unwrap(), TrayEvent::Close);
        drop(rx);
        forward_menu(CLOSE_MENU_ID, &tx);
    }

    #[test]
    fn bootstrap_error_reaches_caller() {
        let (_tx, rx) = async_channel::unbounded();
        let result = start_worker(
            rx,
            || {},
            |ready| {
                ready
                    .send(Err(anyhow::anyhow!("native init failed")))
                    .unwrap();
            },
        );
        assert_eq!(result.err().unwrap().to_string(), "native init failed");
    }

    #[test]
    fn bootstrap_disconnect_reaches_caller() {
        let (_tx, rx) = async_channel::unbounded();
        assert!(start_worker(rx, || {}, |ready| drop(ready)).is_err());
    }

    #[test]
    fn shutdown_and_drop_join_worker() {
        for explicit in [false, true] {
            let (_tx, rx) = async_channel::unbounded();
            let (stop_tx, stop_rx) = mpsc::channel();
            let stopped = Arc::new(AtomicBool::new(false));
            let worker_stopped = stopped.clone();
            let handle = start_worker(
                rx,
                move || {
                    let _ = stop_tx.send(());
                },
                move |ready| {
                    ready.send(Ok(())).unwrap();
                    stop_rx.recv().unwrap();
                    worker_stopped.store(true, Ordering::SeqCst);
                },
            )
            .unwrap();
            if explicit {
                handle.shutdown();
            } else {
                drop(handle);
            }
            assert!(stopped.load(Ordering::SeqCst));
        }
    }

    #[test]
    fn menu_route_releases_channel_and_can_be_replaced() {
        let (tx, rx) = async_channel::unbounded();
        let route = MenuRoute::install(tx).unwrap();
        let (other, _) = async_channel::unbounded();
        assert!(MenuRoute::install(other).is_err());
        drop(route);
        assert!(rx.is_closed());
        let (tx, rx) = async_channel::unbounded();
        let route = MenuRoute::install(tx).unwrap();
        drop(route);
        assert!(rx.is_closed());
    }
}
