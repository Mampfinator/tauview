#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::path::{Path, PathBuf};
use tauri::api::dir::{read_dir, DiskEntry};
use tauri::Manager;

mod menu;

#[derive(Clone, serde::Serialize)]
struct Payload {
    message: String,
}

fn mime_from<P: AsRef<Path>>(filepath: P) -> bool {
    match infer::get_from_path(filepath) {
        Ok(kind) => match kind {
            Some(mime) => {
                mime.mime_type() == "image/jpeg"
                    || mime.mime_type() == "image/png"
                    || mime.mime_type() == "image/gif"
                    || mime.mime_type() == "image/webp"
                    || mime.mime_type() == "image/vnd.microsoft.icon"
            }
            None => false,
        },
        Err(why) => {
            println!("Error in infer::get_from_path: {:?}", why);
            false
        }
    }
}

#[tauri::command]
fn mime_check(filepath: String) -> bool {
    mime_from(filepath)
}

#[tauri::command]
fn move_to_trash(url: String) -> Result<(), String> {
    match trash::delete(url) {
        Ok(_) => Ok(()),
        Err(error) => Err(error.to_string()),
    }
}

fn is_dir(entry: &DiskEntry) -> bool {
    entry.children.is_some()
}

fn is_dot(entry: &DiskEntry) -> bool {
    match &entry.name {
        Some(name) => name.starts_with('.'),
        None => true,
    }
}

fn is_img(entry: &DiskEntry) -> bool {
    mime_from(&entry.path)
}

#[tauri::command]
fn get_entries(dir: String) -> Vec<PathBuf> {
    let entries = match read_dir(dir, false) {
        Err(why) => {
            println!("Error in read_dir(): {:?}", why);
            let vec: Vec<DiskEntry> = Vec::new();
            vec
        }
        Ok(list) => list,
    };

    let mut paths = entries
        .iter()
        .filter(|entry| !is_dir(entry) && !is_dot(entry) && is_img(entry))
        .map(|entry| entry.path.to_path_buf())
        .collect::<Vec<PathBuf>>();

    paths.sort();
    paths
}

fn main() {
    tauri::Builder::default()
        .menu(menu::default())
        .setup(|app| {
            let window = app.get_window("main").unwrap();
            #[cfg(all(debug_assertions, target_os = "macos"))]
            window.open_devtools();

            let window_ = window.clone();
            window.on_menu_event(move |event| match event.menu_item_id() {
                "open" => window_
                    .emit(
                        "open",
                        Payload {
                            message: "open".into(),
                        },
                    )
                    .expect("Error while emitting open event"),
                "close" => std::process::exit(0),
                "minimize" => {
                    if let Err(why) = window_.minimize() {
                        println!("Error: {:?}", why)
                    }
                }
                "zoom" => match window_.is_maximized() {
                    Ok(result) => {
                        if result {
                            match window_.unmaximize() {
                                Ok(_) => (),
                                Err(why) => println!("Error: {:?}", why),
                            }
                        } else {
                            match window_.maximize() {
                                Ok(_) => (),
                                Err(why) => println!("Error: {:?}", why),
                            }
                        }
                    }
                    Err(why) => println!("Error {:?}", why),
                },
                "fullscreen" => match window_.is_fullscreen() {
                    Ok(result) => {
                        if result {
                            match window_.set_fullscreen(false) {
                                Ok(_) => (),
                                Err(why) => println!("Error: {:?}", why),
                            }
                        } else {
                            match window_.set_fullscreen(true) {
                                Ok(_) => (),
                                Err(why) => println!("Error: {:?}", why),
                            }
                        }
                    }
                    Err(why) => println!("Error: {:?}", why),
                },
                _ => {}
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            move_to_trash,
            mime_check,
            get_entries
        ])
        .plugin(tauri_plugin_window_state::WindowState::default())
        .run(tauri::generate_context!())
        .expect("Error while running tauri application");
}
