// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod convert_audio;

use std::sync::Mutex;

use tauri::Manager;
use vvcapi::{VoicevoxCore, InitializeOptions};

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            // Voicevox Coreインスタンス
            let core = VoicevoxCore::new(
                InitializeOptions {
                    open_jtalk_dict_dir: "./open_jtalk_dic_utf_8-1.11".to_string(),
                    ..Default::default()
                }
            ).expect("failed to initialize VoicevoxCore");
            println!("Voicevox Core version: {}", core.get_version());

            app.manage(core);

            // 生成のキャンセルフラグ
            let cancel_flag = Mutex::new(false);
            app.manage(cancel_flag);

            // キャンセルeventのlistener
            let handle = app.handle();
            let _id = app.listen_global("cancel-request", move |_event| {
                handle.emit_all("progress", "キャンセル中...").unwrap();
                let cancel_flag = handle.state::<Mutex<bool>>();
                let mut cancel_flag = cancel_flag.lock().unwrap();
                *cancel_flag = true;
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            commands::get_default_path,
            commands::generate_audio
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
