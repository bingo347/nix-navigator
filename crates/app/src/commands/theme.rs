use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Default, Clone, Copy)]
#[repr(u8)]
pub enum Theme {
    #[default]
    System = 0,
    Light = 1,
    Dark = 2,
}

// TODO: use normal config
mod config {
    use super::Theme;
    use std::sync::atomic::{AtomicU8, Ordering};

    static THEME: AtomicU8 = AtomicU8::new(Theme::System as u8);

    pub fn set(theme: Theme) {
        THEME.store(theme as u8, Ordering::SeqCst);
    }

    pub fn get() -> Theme {
        Theme::from(THEME.load(Ordering::SeqCst))
    }
}

#[tauri::command]
pub fn themeSet(theme: Theme) {
    config::set(theme);
}

#[tauri::command]
pub fn themeGet() -> Theme {
    config::get()
}

impl Serialize for Theme {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u8(*self as u8)
    }
}

impl<'de> Deserialize<'de> for Theme {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = u8::deserialize(deserializer)?;
        Ok(value.into())
    }
}

impl From<u8> for Theme {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::System,
            1 => Self::Light,
            2 => Self::Dark,
            _ => Self::default(),
        }
    }
}
