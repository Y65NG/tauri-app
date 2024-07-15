// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use clipboard::{ClipboardContext, ClipboardProvider};
use enigo::{Enigo, Keyboard, Mouse};
use rdev::{simulate, Button, EventType};
use serde::Serialize;
use std::{
    sync::mpsc::{channel, Sender},
    thread, time,
};
use tauri::{App, Manager};

use webbrowser;

const WINDOW_WIDTH: i32 = 300;
const WINDOW_HEIGHT: i32 = 80;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command

#[tauri::command]
fn get_clipboard() -> String {
    let mut clipboard_ctx: ClipboardContext = ClipboardProvider::new().unwrap();
    if let Ok(content) = clipboard_ctx.get_contents() {
        content
    } else {
        "".to_string()
    }
}

#[tauri::command]
fn copy_to_clipboard() {
    let mut enigo = Enigo::new(&enigo::Settings::default()).unwrap();
    enigo.key(enigo::Key::RCommand, enigo::Direction::Release);
    enigo.key(enigo::Key::Unicode('c'), enigo::Direction::Release);

    enigo.key(enigo::Key::RCommand, enigo::Direction::Press);
    // thread::sleep(time::Duration::from_millis(20));
    enigo.key(enigo::Key::Unicode('c'), enigo::Direction::Press);
    // thread::sleep(time::Duration::from_millis(20));
    enigo.key(enigo::Key::Unicode('c'), enigo::Direction::Release);
    enigo.key(enigo::Key::RCommand, enigo::Direction::Release);
    // send(&EventType::KeyPress(Key::MetaLeft));
    // send(&EventType::KeyPress(Key::KeyC));
}

#[tauri::command]
fn set_clipboard(text: String) {
    let mut clipboard_ctx: ClipboardContext = ClipboardProvider::new().unwrap();
    clipboard_ctx.set_contents(text);
}

#[tauri::command]
fn open_url_in_default_browser(url: String) {
    if let Err(e) = webbrowser::open(&url) {
        eprintln!("Failed to open URL in default browser: {}", e);
    }
}

#[derive(Serialize, Clone)]
struct Position {
    x: i32,
    y: i32,
}

fn start_keyboard_listener(app: &mut App) {
    let handle = app.handle();
    std::thread::spawn(|| {
        let mut enigo = Enigo::new(&enigo::Settings::default()).unwrap();
        let _ = rdev::listen(move |event| {
            match event.event_type {
                EventType::ButtonPress(button) => {
                    if button == Button::Left {
                        let (x, y) = Enigo::location(&enigo).unwrap();
                        let (win_x, win_y) = (x - WINDOW_WIDTH / 2, y - WINDOW_HEIGHT - 20);
                        handle.emit_all("left-click", Position { x, y }).unwrap();
                    }
                }

                EventType::ButtonRelease(button) => {
                    if button == Button::Left {
                        let (x, y) = Enigo::location(&enigo).unwrap();

                        handle
                            .emit_all("left-click-release", Position { x, y })
                            .unwrap();
                    }
                }
                EventType::KeyPress(key) => {
                    if key == rdev::Key::Escape {
                        handle.hide();
                    }
                }
                _ => (),
            }
            // handle.emit_all("click", ()).unwrap();
        });
    });
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            start_keyboard_listener(app);
            let handle = app.handle();
            // hide app window on startup
            handle.get_window("main").unwrap().hide();

            let window = handle.get_window("main").unwrap();
            window.set_always_on_top(true);
            let window_handle = window.clone();
            window.on_window_event(move |e| {
                if let tauri::WindowEvent::Focused(focused) = e {
                    if !focused {
                        window_handle.hide();
                    }
                }
            });
            app.listen_global("selecting", move |event| {
                let window = handle.get_window("main").unwrap();
                window.show();
                window.set_focus();
                let payload = event.payload().unwrap();
                let payload = payload
                    .chars()
                    .skip(1)
                    .take_while(|c| *c != ']')
                    .collect::<String>();
                let payload: Vec<&str> = payload.split(",").collect();
                let payload: Vec<i32> = payload.iter().map(|s| s.parse().unwrap()).collect();
                let (x, y) = (payload[0], payload[1]);
                let (win_x, win_y) = (x - WINDOW_WIDTH / 2, y - WINDOW_HEIGHT - 20);

                handle
                    .get_window("main")
                    .unwrap()
                    .set_position(tauri::Position::Logical(tauri::LogicalPosition {
                        x: win_x as f64,
                        y: win_y as f64,
                    }))
                    .unwrap();
                // window.set_position()
            });

            let handle = app.handle();
            app.listen_global("not-selecting", move |event| {
                window.hide();
            });

            // app.listen_global("tauri://blur" )
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            set_clipboard,
            get_clipboard,
            copy_to_clipboard,
            open_url_in_default_browser,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
