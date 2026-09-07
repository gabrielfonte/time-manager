mod tray;
mod gui;
use crate::tray::tray::Tray;
use crate::gui::gui::{Gui, IcedGui};
use anyhow::Result;

fn main() -> Result<()> {
    // Initialize the system tray
    let system_tray_handle = Tray::init()?;
    println!("Tray initialized. Starting GUI.");

    // Initialize the GUI
    IcedGui::init()?;

    // Clean up the system tray when the GUI exits
    system_tray_handle.shutdown();
    Ok(())
}
