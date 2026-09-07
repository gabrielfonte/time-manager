
use anyhow::Result;
use super::timer::{Message, ProjectTimer};

pub trait Gui {
    fn init() -> Result<()>;
}

pub struct IcedGui;

impl Gui for IcedGui {
    fn init() -> Result<()> {
        iced::application(ProjectTimer::default, update, ProjectTimer::view)
            .subscription(ProjectTimer::subscription)
            .run()?;

        Ok(())
    }
}

fn update(timer: &mut ProjectTimer, message: Message) -> iced::Task<Message> {
    match message {
        Message::TrayPoll => match tray_icon::menu::MenuEvent::receiver().try_recv() {
            Ok(event) if event.id() == crate::tray::tray::OPEN_MENU_ID => {
                iced::window::oldest().then(|window_id| {
                    window_id.map_or_else(iced::Task::none, |id| {
                        iced::window::set_mode::<Message>(id, iced::window::Mode::Windowed)
                            .then(move |_: Message| iced::window::gain_focus(id))
                    })
                })
            }
            Ok(event) if event.id() == crate::tray::tray::CLOSE_MENU_ID => iced::exit(),
            _ => iced::Task::none(),
        },
        message => {
            timer.update(message);
            iced::Task::none()
        }
    }
}
