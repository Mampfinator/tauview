#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::env::consts;
use std::path::PathBuf;
use tauri::api::{dialog, shell};
use tauri::{Manager, Menu};

mod cmd;
mod menu;

#[derive(Clone, serde::Serialize)]
struct Payload {
    message: Option<PathBuf>,
}

fn main() {
    let context = tauri::generate_context!();
    tauri::Builder::default()
        .menu(if consts::OS == "windows" {
            Menu::new()
        } else {
            menu::default(&context)
        })
        .on_menu_event(|event| match event.menu_item_id() {
            "open" => dialog::FileDialogBuilder::new()
                .add_filter("Image File", &["ico", "gif", "png", "jpg", "jpeg", "webp"])
                .pick_file(move |f| {
                    event
                        .window()
                        .emit("open", Payload { message: f })
                        .expect("Error while emitting open event")
                }),
            "close" => std::process::exit(0),
            "next" => event
                .window()
                .emit("menu-next", ())
                .expect("Error while emitting next event"),
            "prev" => event
                .window()
                .emit("menu-prev", ())
                .expect("Error while emitting prev event"),
            "grid" => event
                .window()
                .emit("menu-grid", ())
                .expect("Error while emitting grid event"),
            "remove" => event
                .window()
                .emit("menu-remove", ())
                .expect("Error while emitting remove event"),
            "minimize" => event.window().minimize().unwrap(),
            "zoom" => {
                if let Ok(result) = event.window().is_maximized() {
                    if result {
                        event.window().unmaximize().unwrap();
                    } else {
                        event.window().maximize().unwrap();
                    }
                }
            }
            "fullscreen" => {
                if let Ok(result) = event.window().is_fullscreen() {
                    if result {
                        event.window().set_fullscreen(false).unwrap();
                    } else {
                        event.window().set_fullscreen(true).unwrap();
                    }
                }
            }
            "support" => shell::open(
                &event.window().shell_scope(),
                "https://github.com/sprout2000/tauview#readme",
                None,
            )
            .expect("Error while opening external URL"),
            _ => {}
        })
        .setup(|app| {
            let window = app.get_window("main").unwrap();
            #[cfg(debug_assertions)]
            window.open_devtools();


            if let Some(path) = std::env::args().nth(1) {
                // we get two references to the same Window here since we can't move `window` into the closure.
                let main_window = app.get_window("main").unwrap();

                window.once("app-ready", move |_| {
                    main_window
                        .emit("open", Payload { message: Some(path.into()) })
                        .expect("Error while emitting open event");
                });
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            cmd::mime_check,
            cmd::open_dialog,
            cmd::get_entries,
            cmd::move_to_trash,
        ])
        .on_window_event(|event|  {
            if let tauri::WindowEvent::FileDrop(tauri::FileDropEvent::Dropped(paths)) = event.event() {
                event
                    .window()
                    .emit("open", Payload { message: Some(paths[0].clone()) })
                    .expect("Error while emitting open event");
            }
        })
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .run(context)
        .expect("Error while running tauri application");
}
