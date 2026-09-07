#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod tray;
mod gui;

use anyhow::Result;
use crate::tray::tray::Tray;
use crate::gui::gui::{Gui, IcedGui};

fn main() -> Result<()> {
    // Initialize the system tray
    let tray = Tray::init()?;
    // Initialize the GUI (Pass the tray event receiver to the GUI)
    let result = IcedGui::init(tray.events());
    // Clean up the system tray when the GUI exits
    tray.shutdown();
    Ok(())
}