#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use crate::commands::BuilderExt as _;

mod commands;
mod menu;

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            menu::setup_menu(app)?;
            Ok(())
        })
        .with_commands()
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
