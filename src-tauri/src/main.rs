// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use cignaler::database::database::{
    delete_ci_server_data, delete_project_data, init_db, read_ci_servers_data,
    read_projects_data, save_ci_server_data, save_project_data, update_ci_server_data,
    update_project_data, update_project_enabled,
};
use cignaler::gitlab_client::gitlab_client::{get_gitlab_pipelines, get_references, PipelineData};
use cignaler::{CiProject, CiServer};
use tauri::{
    image::Image,
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager,
};
use tracing::{debug, error, info};

#[tauri::command]
fn get_pipelines(
    ci_server_name: String,
    project_name: String,
    reference: String,
) -> Result<Vec<PipelineData>, String> {
    debug!(
        "Getting pipelines for server: {}, project: {}, ref: {}",
        ci_server_name, project_name, reference
    );

    let servers = read_ci_servers_data().map_err(|e| e.to_string())?;

    let ci_server = servers
        .iter()
        .find(|server| server.name == ci_server_name)
        .ok_or_else(|| format!("CI server '{}' not found", ci_server_name))?;

    if ci_server.server_type != "gitlab" {
        return Err(format!(
            "Server type '{}' not supported yet",
            ci_server.server_type
        ));
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
    debug!(
        "Saving CI server data: name={}, type={}, url={}",
        name, server_type, url_string
    );
    save_ci_server_data(name, server_type, url_string, api_key).map_err(|e| e.to_string())
}

#[tauri::command]
fn read_ci_servers() -> Result<Vec<CiServer>, String> {
    let servers = read_ci_servers_data().map_err(|e| e.to_string())?;
    debug!("Successfully loaded {} CI servers", servers.len());
    Ok(servers)
}

#[tauri::command]
fn update_ci_server(
    name: String,
    server_type: String,
    url_string: String,
    api_key: String,
) -> Result<(), String> {
    debug!(
        "Updating CI server: name={}, type={}, url={}",
        name, server_type, url_string
    );
    update_ci_server_data(name, server_type, url_string, api_key).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_ci_server(name: String) -> Result<(), String> {
    debug!("Deleting CI server: name={}", name);
    delete_ci_server_data(name).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_pipeline_references(
    ci_server_name: String,
    project_name: String,
) -> Result<Vec<String>, String> {
    debug!(
        "Getting references for server: {}, project: {}",
        ci_server_name, project_name
    );

    let servers = read_ci_servers_data().map_err(|e| e.to_string())?;

    let ci_server = servers
        .iter()
        .find(|server| server.name == ci_server_name)
        .ok_or_else(|| format!("CI server '{}' not found", ci_server_name))?;

    if ci_server.server_type != "gitlab" {
        return Err(format!(
            "Server type '{}' not supported yet",
            ci_server.server_type
        ));
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
    debug!(
        "Saving project data: name={}, server={}, path={}",
        name, ci_server_name, project_path
    );
    save_project_data(name, ci_server_name, project_path, default_branch).map_err(|e| e.to_string())
}

#[tauri::command]
fn read_projects() -> Result<Vec<CiProject>, String> {
    let projects = read_projects_data().map_err(|e| e.to_string())?;
    debug!("Successfully loaded {} projects", projects.len());
    Ok(projects)
}

#[tauri::command]
fn update_project(
    id: i64,
    name: String,
    ci_server_name: String,
    project_path: String,
    default_branch: Option<String>,
) -> Result<(), String> {
    debug!(
        "Updating project: id={}, name={}, server={}, path={}",
        id, name, ci_server_name, project_path
    );
    update_project_data(id, name, ci_server_name, project_path, default_branch)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn set_project_enabled(id: i64, enabled: bool) -> Result<(), String> {
    debug!("Setting project enabled: id={}, enabled={}", id, enabled);
    update_project_enabled(id, enabled).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_project(id: i64) -> Result<(), String> {
    debug!("Deleting project: id={}", id);
    delete_project_data(id).map_err(|e| e.to_string())
}

#[tauri::command]
fn update_tray_icon(app: AppHandle, state: String) -> Result<(), String> {
    debug!("Updating tray icon to state: {}", state);

    let icon_filename = match state.as_str() {
        "success" => "tray-success.png",
        "failed" => "tray-failed.png",
        _ => "tray-pending.png",
    };

    let resource_dir = app
        .path()
        .resource_dir()
        .map_err(|e| format!("Failed to get resource dir: {}", e))?;

    let icon_path = resource_dir.join("icons").join(icon_filename);
    debug!("Loading icon from: {:?}", icon_path);

    let icon = Image::from_path(&icon_path)
        .map_err(|e| format!("Failed to load icon from {:?}: {}", icon_path, e))?;

    if let Some(tray) = app.tray_by_id("main-tray") {
        tray.set_icon(Some(icon))
            .map_err(|e| format!("Failed to set tray icon: {}", e))?;
        debug!("Tray icon updated successfully to: {}", state);
        Ok(())
    } else {
        Err("Tray icon with ID 'main-tray' not found".to_string())
    }
}

fn main() {
    // Initialize tracing subscriber for structured logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("cignaler=debug".parse().unwrap())
                .add_directive("r2d2=warn".parse().unwrap()),
        )
        .init();

    info!("Starting Cignaler...");

    // Initialize database - exit on failure
    if let Err(e) = init_db() {
        error!("Failed to initialize database: {}", e);
        std::process::exit(1);
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            get_pipelines,
            store_ci_server_data,
            update_ci_server,
            delete_ci_server,
            read_ci_servers,
            get_pipeline_references,
            store_project_data,
            read_projects,
            update_project,
            set_project_enabled,
            delete_project,
            update_tray_icon
        ])
        .setup(|app| {
            let toggle = MenuItemBuilder::with_id("toggle", "Show/Hide").build(app)?;
            let quit = MenuItemBuilder::with_id("quit", "Quit").build(app)?;
            let menu = MenuBuilder::new(app).items(&[&toggle, &quit]).build()?;

            let _tray = TrayIconBuilder::with_id("main-tray")
                .menu(&menu)
                .menu_on_left_click(false)
                .on_menu_event(move |app, event| match event.id().as_ref() {
                    "toggle" => {
                        debug!("Toggle menu clicked");
                        if let Some(window) = app.get_webview_window("main") {
                            if window.is_visible().unwrap_or(false) {
                                let _ = window.hide();
                            } else {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                    }
                    "quit" => {
                        info!("Quit requested from tray menu");
                        std::process::exit(0);
                    }
                    _ => (),
                })
                .on_tray_icon_event(|tray, event| match event {
                    TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } => {
                        debug!("Tray icon left click");
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
                        debug!("Tray icon right click");
                    }
                    _ => (),
                })
                .icon(app.default_window_icon().unwrap().clone())
                .build(app)?;

            // Set up window close handler to minimize to tray instead of closing
            let main_window = app.get_webview_window("main");
            if let Some(window) = main_window {
                let window_clone = window.clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        debug!("Window close requested - hiding to tray");
                        api.prevent_close();
                        let _ = window_clone.hide();
                    }
                });
            }

            info!("Application setup complete");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application")
}
