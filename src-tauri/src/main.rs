// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use ::gitlab::api::Query;
use gitlab::{api, Gitlab};
use std::env::consts::OS;
use std::fs::File;
use std::time::{SystemTime};
use serde::{Deserialize, Serialize};
use serde_json::to_string;
use tauri::{CustomMenuItem, SystemTray, SystemTrayMenu, SystemTrayMenuItem};

use crate::database::database::init_db;
use crate::gitlab_client::gitlab_client::{get_gitlab_pipelines, get_references, PipelineData};

mod database;
mod gitlab_client;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn get_pipelines() -> Result<Vec<PipelineData>, String> {
    let client = Gitlab::new("gitlab.com", "glpat-hFFsfxjumhsB9W-nor8Q");
    if client.is_err() {
        let error = client.err().unwrap();
        eprintln!("{}", error.to_string());
        return Err(error.to_string())
    }

    let client = client.unwrap();
    let data = get_gitlab_pipelines("preview", "flipcorp/flipnext/flip", &client);
    return Ok(data)
}

#[tauri::command]
async fn background_task() {
    loop {
        File::create("lala.txt").unwrap();
        std::thread::sleep(std::time::Duration::from_secs(5));
    }
}

#[tauri::command]
fn get_pipeline_references() -> Vec<String> {
    println!("get_references");
    let client = Gitlab::new("gitlab.com", "glpat-ACsoJqTaSSakzcyJnbcz");
    if client.is_err() {
        eprintln!("{}", client.err().unwrap());
        return vec![]
    }

    let client = client.unwrap();
    let refs = get_references("flipcorp/flipnext/flip", &client);
    return refs
}

fn main() {
    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    let hide = CustomMenuItem::new("hide".to_string(), "Hide");
    let init_success = init_db();
    if init_success.is_err() { eprintln!("{}", init_success.err().unwrap()) }
    let tray_menu = SystemTrayMenu::new()
        .add_item(quit)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(hide);
    let system_tray = SystemTray::new()
        .with_menu(tray_menu);
    tauri::Builder::default()
        .system_tray(system_tray)
        .invoke_handler(tauri::generate_handler![greet, get_pipelines, background_task])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
