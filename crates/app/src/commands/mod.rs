#![allow(non_snake_case)]

use tauri::{Builder, Runtime};

mod theme;

pub trait BuilderExt {
    fn with_commands(self) -> Self;
}

impl<R: Runtime> BuilderExt for Builder<R> {
    fn with_commands(self) -> Self {
        self.invoke_handler(tauri::generate_handler![theme::themeGet, theme::themeSet])
    }
}
