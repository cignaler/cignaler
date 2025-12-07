// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use crate::database::database::{init_db, read_ci_servers_data, save_ci_server_data, update_ci_server_data, save_project_data, read_projects_data};
use crate::gitlab_client::gitlab_client::{get_gitlab_pipelines, get_references, PipelineData};
use cignaler::{CiServer, CiProject};
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
fn get_pipelines(
    ci_server_name: String,
    project_name: String,
    reference: String,
) -> Result<Vec<PipelineData>, String> {
    println!("Getting pipelines for server: {}, project: {}, ref: {}", ci_server_name, project_name, reference);
    
    // Get CI server configuration from database
    let servers = match read_ci_servers_data() {
        Ok(servers) => servers,
        Err(e) => return Err(format!("Failed to read CI servers: {}", e)),
    };

    let ci_server = servers
        .iter()
        .find(|server| server.name == ci_server_name)
        .ok_or_else(|| format!("CI server '{}' not found", ci_server_name))?;

    // Only support GitLab for now
    if ci_server.server_type != "gitlab" {
        return Err(format!("Server type '{}' not supported yet", ci_server.server_type));
    }

    get_gitlab_pipelines(&reference, &project_name, ci_server)
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
fn read_ci_servers() -> Result<Vec<CiServer>, String> {
    match read_ci_servers_data() {
        Ok(data) => {
            println!("Successfully loaded {} CI servers", data.len());
            Ok(data)
        }
        Err(e) => {
            println!("Failed to read CI servers: {}", e);
            Err(format!("Failed to read CI servers: {}", e))
        }
    }
}

#[tauri::command]
fn update_ci_server(
    name: String,
    server_type: String,
    url_string: String,
    api_key: String,
) -> Result<(), String> {
    println!("Updating CI server: name={}, type={}, url={}", name, server_type, url_string);
    match update_ci_server_data(name, server_type, url_string, api_key) {
        Ok(_) => {
            println!("CI server updated successfully");
            Ok(())
        }
        Err(e) => {
            println!("Failed to update CI server: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
fn get_pipeline_references(
    ci_server_name: String,
    project_name: String,
) -> Result<Vec<String>, String> {
    println!("Getting references for server: {}, project: {}", ci_server_name, project_name);
    
    // Get CI server configuration from database
    let servers = match read_ci_servers_data() {
        Ok(servers) => servers,
        Err(e) => return Err(format!("Failed to read CI servers: {}", e)),
    };

    let ci_server = servers
        .iter()
        .find(|server| server.name == ci_server_name)
        .ok_or_else(|| format!("CI server '{}' not found", ci_server_name))?;

    // Only support GitLab for now
    if ci_server.server_type != "gitlab" {
        return Err(format!("Server type '{}' not supported yet", ci_server.server_type));
    }

    get_references(&project_name, ci_server)
}

#[tauri::command]
fn store_project_data(
    name: String,
    ci_server_name: String,
    project_path: String,
    default_branch: Option<String>,
) -> Result<(), String> {
    println!("Saving project data: name={}, server={}, path={}", name, ci_server_name, project_path);
    match save_project_data(name, ci_server_name, project_path, default_branch) {
        Ok(_) => {
            println!("Project data saved successfully");
            Ok(())
        }
        Err(e) => {
            println!("Failed to save project data: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
fn read_projects() -> Result<Vec<CiProject>, String> {
    match read_projects_data() {
        Ok(data) => {
            println!("Successfully loaded {} projects", data.len());
            Ok(data)
        }
        Err(e) => {
            println!("Failed to read projects: {}", e);
            Err(format!("Failed to read projects: {}", e))
        }
    }
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
            update_ci_server,
            read_ci_servers,
            get_pipeline_references,
            store_project_data,
            read_projects
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
