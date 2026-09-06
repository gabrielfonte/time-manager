mod tray;
mod gui;
use crate::tray::tray::GtkTray;
use crate::gui::gui::{Gui, IcedGui};
use anyhow::Result;

fn main() -> Result<()> {
    let tray_handle = GtkTray::init()?;
    println!("Tray initialized. Starting GUI.");
    IcedGui::init()?;
    tray_handle.shutdown();
    Ok(())
}
