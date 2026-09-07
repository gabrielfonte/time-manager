mod tray;
mod gui;
use crate::tray::tray::Tray;
use crate::gui::gui::{Gui, IcedGui};
use anyhow::Result;

fn main() -> Result<()> {
    let tray_handle = Tray::init()?;
    println!("Tray initialized. Starting GUI.");
    IcedGui::init()?;
    tray_handle.shutdown();
    Ok(())
}
