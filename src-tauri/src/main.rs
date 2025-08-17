// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use crate::database::database::{init_db, read_ci_servers_data, save_ci_server_data};
use crate::gitlab_client::gitlab_client::{get_gitlab_pipelines, get_references, PipelineData};
use ::gitlab::api::Query;
use cignaler::CiServer;
use gitlab::{api, Gitlab};
use serde::ser::Error;
use serde::{Deserialize, Serialize};
use serde_json::to_string;
use std::env::consts::OS;
use std::fmt::write;
use std::fs::File;
use std::time::SystemTime;
use tauri::tray;
use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager,
};

mod database;
mod gitlab_client;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn get_pipelines() -> Result<Vec<PipelineData>, String> {
    let client = Gitlab::new("gitlab.com", "glpat-p3rSwf2m2Ux2U4bnEufn");
    if client.is_err() {
        let error = client.err().unwrap();
        eprintln!("{}", error.to_string());
        return Err(error.to_string());
    }

    let client = client.unwrap();
    let data = get_gitlab_pipelines("preview", "flipcorp/flipnext/flip", &client);
    return Ok(data);
}

#[tauri::command]
async fn background_task() {
    loop {
        File::create("lala.txt").unwrap();
        std::thread::sleep(std::time::Duration::from_secs(5));
    }
}

#[tauri::command]
fn store_ci_server_data(
    name: String,
    server_type: String,
    url_string: String,
    api_key: String,
) -> Result<(), String> {
    println!("saving CI server data: name={}, type={}, url={}", name, server_type, url_string);
    match save_ci_server_data(name, server_type, url_string, api_key) {
        Ok(_) => {
            println!("CI server data saved successfully");
            Ok(())
        }
        Err(e) => {
            println!("Failed to save CI server data: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
fn read_ci_servers() -> Vec<CiServer> {
    let data = read_ci_servers_data().unwrap();
    println!("data: {:?}", data);
    data
}

#[tauri::command]
fn get_pipeline_references() -> Vec<String> {
    println!("get_references");
    let client = Gitlab::new("gitlab.com", "glpat-ACsoJqTaSSakzcyJnbcz");
    if client.is_err() {
        eprintln!("{}", client.err().unwrap());
        return vec![];
    }

    let client = client.unwrap();
    let refs = get_references("flipcorp/flipnext/flip", &client);
    return refs;
}

fn main() {
    let init_success = init_db();
    if init_success.is_err() {
        eprintln!("{}", init_success.err().unwrap())
    }
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            get_pipelines,
            store_ci_server_data,
            read_ci_servers,
            background_task
        ])
        .setup(|app| {
            let toggle = MenuItemBuilder::with_id("toggle", "Toggle").build(app)?;
            let hide = MenuItemBuilder::with_id("hide", "Hide").build(app)?;
            let quit = MenuItemBuilder::with_id("quit", "Quit").build(app)?;
            let menu = MenuBuilder::new(app)
                .items(&[&toggle, &hide, &quit])
                .build()?;
            let tray = TrayIconBuilder::new()
                .menu(&menu)
                .menu_on_left_click(true)
                .on_menu_event(move |app, event| match event.id().as_ref() {
                    "toggle" => {
                        println!("toggle clicked");
                    }
                    "quit" => std::process::exit(0),
                    _ => (),
                })
                .on_tray_icon_event(|tray, event| match event {
                    TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } => {
                        println!("left click pressed and released");
                        let app = tray.app_handle();
                        if let Some(webview_window) = app.get_webview_window("main") {
                            let _ = webview_window.show();
                            let _ = webview_window.set_focus();
                        }
                    }
                    TrayIconEvent::Click {
                        button: MouseButton::Right,
                        button_state: MouseButtonState::Up,
                        ..
                    } => {
                        println!("right click pressed");
                    }
                    _ => (),
                })
                .icon(app.default_window_icon().unwrap().clone())
                .build(app)?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application")
}
