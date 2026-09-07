#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod tray;
mod gui;
use crate::tray::tray::Tray;
use crate::gui::gui::{Gui, IcedGui};
use anyhow::Result;

fn main() -> Result<()> {
    // Initialize the system tray
    let mut system_tray_handle = Tray::init()?;
    // Initialize the GUI (Pass the tray event receiver to the GUI)
    IcedGui::init(system_tray_handle.take_events())?;

    // Clean up the system tray when the GUI exits
    system_tray_handle.shutdown();
    Ok(())
}
