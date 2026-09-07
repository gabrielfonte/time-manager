
use anyhow::Result;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use super::timer::{Message, ProjectTimer};
use crate::tray::tray::TrayEvent;

pub trait Gui {
    fn init(tray_events: Receiver<TrayEvent>) -> Result<()>;
}

pub struct IcedGui;

impl Gui for IcedGui {
    fn init(tray_events: Receiver<TrayEvent>) -> Result<()> {
        let tray_events = Arc::new(Mutex::new(Some(tray_events)));
        iced::application(
            move || ProjectTimer::new(tray_events.lock().unwrap().take().unwrap()),
            update,
            ProjectTimer::view,
            )
            .window(iced::window::Settings {
                icon: Some(window_icon()?),
                ..Default::default()
            })
            .exit_on_close_request(false)
            .subscription(ProjectTimer::subscription)
            .run()?;

        Ok(())
    }
}

fn window_icon() -> Result<iced::window::Icon> {
    let icon_bytes = include_bytes!("../tray/icon.ico");
    let image = image::load_from_memory(icon_bytes)?.into_rgba8();
    let (width, height) = image.dimensions();

    Ok(iced::window::icon::from_rgba(
        image.into_raw(),
        width,
        height,
    )?)
}

fn update(timer: &mut ProjectTimer, message: Message) -> iced::Task<Message> {
    match message {
        Message::WindowCloseRequested => iced::window::oldest().then(|window_id| {
            window_id.map_or_else(iced::Task::none, |id| {
                iced::window::minimize(id, true)
            })
        }),
        Message::TrayPoll => match timer.try_tray_event() {
            Some(TrayEvent::Close) => iced::exit(),
            None => iced::Task::none(),
        },
        message => {
            timer.update(message);
            iced::Task::none()
        }
    }
}
