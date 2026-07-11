#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use crate::commands::BuilderExt as _;

mod commands;

fn main() {
    tauri::Builder::default()
        .with_commands()
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
