
use anyhow::Result;
use super::timer::ProjectTimer;

pub trait Gui {
    fn init() -> Result<()>;
}

pub struct IcedGui;

impl Gui for IcedGui {
    fn init() -> Result<()> {
        iced::application(ProjectTimer::default, ProjectTimer::update, ProjectTimer::view)
            .subscription(ProjectTimer::subscription)
            .run()?;

        Ok(())
    }
}
