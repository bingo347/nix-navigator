use tauri::{
    App, Runtime,
    menu::{
        AboutMetadata, IconMenuItem, Menu, NativeIcon, PredefinedMenuItem, Submenu, SubmenuBuilder,
    },
};
const APP_NAME: &str = "Nix Navigator";

#[cfg(not(target_os = "macos"))]
pub fn setup_menu<R: Runtime>(_app: &App<R>) -> tauri::Result<()> {
    Ok(())
}

#[cfg(target_os = "macos")]
pub fn setup_menu<R: Runtime>(app: &App<R>) -> tauri::Result<()> {
    let handle = app.handle();

    let pkg_info = handle.package_info();
    let config = handle.config();
    let about_metadata = AboutMetadata {
        name: Some(APP_NAME.into()),
        version: Some(pkg_info.version.to_string()),
        copyright: config.bundle.copyright.clone(),
        authors: config.bundle.publisher.clone().map(|p| vec![p]),
        ..Default::default()
    };

    let menu = Menu::with_items(
        handle,
        &[
            &Submenu::with_items(
                handle,
                APP_NAME,
                true,
                &[
                    &SubmenuBuilder::new(handle, "Settings")
                        .submenu_native_icon(NativeIcon::Advanced)
                        .enabled(true)
                        .items(&[
                            &IconMenuItem::with_id_and_native_icon(
                                handle,
                                "config",
                                "Configuration",
                                true,
                                Some(NativeIcon::PreferencesGeneral),
                                Some("CMD+,"),
                            )?,
                            &IconMenuItem::with_id_and_native_icon(
                                handle,
                                "keybindings",
                                "Keybindings",
                                true,
                                Some(NativeIcon::ListView),
                                Some("CMD+K"),
                            )?,
                        ])
                        .build()?,
                    &PredefinedMenuItem::separator(handle)?,
                    &PredefinedMenuItem::services(handle, None)?,
                    &PredefinedMenuItem::separator(handle)?,
                    &PredefinedMenuItem::hide(handle, None)?,
                    &PredefinedMenuItem::hide_others(handle, None)?,
                    &PredefinedMenuItem::separator(handle)?,
                    &PredefinedMenuItem::quit(handle, None)?,
                ],
            )?,
            &Submenu::with_items(
                handle,
                "Edit",
                true,
                &[
                    &PredefinedMenuItem::undo(handle, None)?,
                    &PredefinedMenuItem::redo(handle, None)?,
                    &PredefinedMenuItem::separator(handle)?,
                    &PredefinedMenuItem::cut(handle, None)?,
                    &PredefinedMenuItem::copy(handle, None)?,
                    &PredefinedMenuItem::paste(handle, None)?,
                    &PredefinedMenuItem::select_all(handle, None)?,
                ],
            )?,
            &Submenu::with_items(
                handle,
                "View",
                true,
                &[&PredefinedMenuItem::fullscreen(handle, None)?],
            )?,
            &Submenu::with_id_and_items(
                handle,
                "window",
                "Window",
                true,
                &[
                    &PredefinedMenuItem::minimize(handle, None)?,
                    &PredefinedMenuItem::maximize(handle, None)?,
                    &PredefinedMenuItem::separator(handle)?,
                    &PredefinedMenuItem::close_window(handle, None)?,
                ],
            )?,
            &Submenu::with_id_and_items(
                handle,
                "help",
                "Help",
                true,
                &[&PredefinedMenuItem::about(
                    handle,
                    Some("About Nix Navigator"),
                    Some(about_metadata),
                )?],
            )?,
        ],
    )?;

    app.set_menu(menu)?;
    Ok(())
}
