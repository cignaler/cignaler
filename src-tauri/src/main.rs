// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use cignaler::database::database::{
    delete_ci_server_data, delete_project_data, init_db, read_cached_pipelines,
    read_ci_servers_data, read_projects_data, save_ci_server_data, save_project_data,
    update_ci_server_data, update_project_data, update_project_enabled,
};
use cignaler::gitlab_client::gitlab_client::{get_gitlab_pipelines, get_references, PipelineData};
use cignaler::pipeline_cache::{poll_single_watcher, set_tray_icon, start_background_poller, update_tray_from_all_cached};
use cignaler::{CiProject, CiServer};
use serde::Serialize;
use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager,
};
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixListener;
use tracing::{debug, error, info, warn};

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
fn set_project_enabled(app: AppHandle, id: i64, enabled: bool) -> Result<(), String> {
    debug!("Setting project enabled: id={}, enabled={}", id, enabled);
    update_project_enabled(id, enabled).map_err(|e| e.to_string())?;
    update_tray_from_all_cached(&app);
    Ok(())
}

#[tauri::command]
fn delete_project(id: i64) -> Result<(), String> {
    debug!("Deleting project: id={}", id);
    delete_project_data(id).map_err(|e| e.to_string())
}

#[tauri::command]
fn update_tray_icon(app: AppHandle, state: String) -> Result<(), String> {
    debug!("Updating tray icon to state: {}", state);
    set_tray_icon(&app, &state)
}

#[derive(Serialize)]
struct CachedPipelinesResponse {
    watcher_id: i64,
    pipelines: serde_json::Value,
    last_updated: Option<String>,
    error: Option<String>,
}

#[tauri::command]
fn get_cached_pipelines(watcher_id: i64) -> Result<CachedPipelinesResponse, String> {
    debug!("Reading cached pipelines for watcher_id={}", watcher_id);

    match read_cached_pipelines(watcher_id) {
        Ok(Some(row)) => {
            let pipelines: serde_json::Value = serde_json::from_str(&row.pipelines_json)
                .unwrap_or(serde_json::Value::Array(vec![]));
            Ok(CachedPipelinesResponse {
                watcher_id,
                pipelines,
                last_updated: Some(row.last_updated),
                error: row.error,
            })
        }
        Ok(None) => Ok(CachedPipelinesResponse {
            watcher_id,
            pipelines: serde_json::Value::Array(vec![]),
            last_updated: None,
            error: None,
        }),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
fn show_main_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn trigger_poll(app: AppHandle, watcher_id: i64) -> Result<(), String> {
    debug!("Manual poll triggered for watcher_id={}", watcher_id);
    let handle = app.clone();
    tauri::async_runtime::spawn(async move {
        poll_single_watcher(&handle, watcher_id).await;
    });
    Ok(())
}

const CHROME_EXTENSION_ID: &str = "mnpadnofpbfgbomckhchncfeepeooifd";

fn register_native_messaging_host() {
    let host_binary = match std::env::current_exe() {
        Ok(exe) => exe.parent().unwrap_or(&PathBuf::from(".")).join("cignaler-native-host"),
        Err(e) => {
            warn!("Could not resolve current executable path: {}", e);
            return;
        }
    };

    if !host_binary.exists() {
        warn!(
            "Native messaging host binary not found at {:?}, skipping registration",
            host_binary
        );
        return;
    }

    let manifest_dir = match dirs::data_dir() {
        Some(d) => d
            .join("Google")
            .join("Chrome")
            .join("NativeMessagingHosts"),
        None => {
            warn!("Could not resolve data directory for native messaging manifest");
            return;
        }
    };

    if let Err(e) = fs::create_dir_all(&manifest_dir) {
        warn!("Failed to create native messaging manifest directory: {}", e);
        return;
    }

    let manifest = serde_json::json!({
        "name": "com.cignaler.app",
        "description": "Cignaler native messaging host",
        "path": host_binary.to_string_lossy(),
        "type": "stdio",
        "allowed_origins": [format!("chrome-extension://{}/", CHROME_EXTENSION_ID)]
    });

    let manifest_path = manifest_dir.join("com.cignaler.app.json");
    match fs::write(&manifest_path, serde_json::to_string_pretty(&manifest).unwrap()) {
        Ok(()) => info!("Native messaging host manifest written to {:?}", manifest_path),
        Err(e) => warn!("Failed to write native messaging host manifest: {}", e),
    }
}

/// Request from native messaging host
#[derive(Deserialize)]
struct NativeHostRequest {
    action: String,
    name: String,
    project: String,
    #[serde(rename = "ref")]
    reference: String,
    ci_server: String,
}

/// Get the Unix socket path for IPC with native host
fn get_socket_path() -> Option<PathBuf> {
    dirs::data_dir().map(|d| d.join("com.ostwi.dev").join("cignaler.sock"))
}

/// Start the Unix socket listener for native host IPC
fn start_ipc_listener(app_handle: AppHandle) {
    let socket_path = match get_socket_path() {
        Some(p) => p,
        None => {
            warn!("Could not resolve socket path for IPC listener");
            return;
        }
    };

    // Remove existing socket file if it exists
    let _ = fs::remove_file(&socket_path);

    // Ensure parent directory exists
    if let Some(parent) = socket_path.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            warn!("Failed to create socket directory: {}", e);
            return;
        }
    }

    tauri::async_runtime::spawn(async move {
        let listener = match UnixListener::bind(&socket_path) {
            Ok(l) => {
                info!("IPC socket listening at {:?}", socket_path);
                l
            }
            Err(e) => {
                error!("Failed to bind IPC socket: {}", e);
                return;
            }
        };

        loop {
            match listener.accept().await {
                Ok((stream, _)) => {
                    let app = app_handle.clone();
                    tauri::async_runtime::spawn(async move {
                        handle_ipc_connection(stream, app).await;
                    });
                }
                Err(e) => {
                    error!("Failed to accept IPC connection: {}", e);
                }
            }
        }
    });
}

/// Handle a single IPC connection from native host
async fn handle_ipc_connection(stream: tokio::net::UnixStream, app_handle: AppHandle) {
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    // Read JSON line from native host
    match reader.read_line(&mut line).await {
        Ok(0) => return, // Connection closed
        Ok(_) => {}
        Err(e) => {
            error!("Failed to read from IPC socket: {}", e);
            return;
        }
    }

    // Parse request
    let response = match serde_json::from_str::<NativeHostRequest>(&line) {
        Ok(req) => {
            if req.action == "add-watcher" {
                // Save to database
                match save_project_data(
                    req.name.clone(),
                    req.ci_server.clone(),
                    req.project.clone(),
                    Some(req.reference.clone()),
                ) {
                    Ok(()) => {
                        info!("Watcher '{}' added via IPC", req.name);
                        // Emit event to refresh UI
                        if let Err(e) = app_handle.emit("watcher-added", ()) {
                            error!("Failed to emit watcher-added event: {}", e);
                        }
                        r#"{"success":true}"#.to_string()
                    }
                    Err(e) => {
                        error!("Failed to save watcher: {}", e);
                        format!(r#"{{"success":false,"error":"{}"}}"#, e)
                    }
                }
            } else {
                format!(r#"{{"success":false,"error":"Unknown action: {}"}}"#, req.action)
            }
        }
        Err(e) => {
            error!("Failed to parse IPC request: {}", e);
            format!(r#"{{"success":false,"error":"Invalid JSON: {}"}}"#, e)
        }
    };

    // Send response back to native host
    if let Err(e) = writer.write_all(response.as_bytes()).await {
        error!("Failed to write IPC response: {}", e);
    }
    if let Err(e) = writer.write_all(b"\n").await {
        error!("Failed to write IPC newline: {}", e);
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

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
            debug!("Single instance callback: argv={:?}", argv);

            // Show and focus the main window
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
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
            update_tray_icon,
            get_cached_pipelines,
            trigger_poll,
            show_main_window
        ])
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir()
                .map_err(|e| format!("Failed to resolve app data dir: {}", e))?;

            init_db(app_data_dir)
                .map_err(|e| format!("Failed to initialize database: {}", e))?;

            register_native_messaging_host();
            start_ipc_listener(app.handle().clone());

            let toggle = MenuItemBuilder::with_id("toggle", "Show/Hide").build(app)?;
            let quit = MenuItemBuilder::with_id("quit", "Quit").build(app)?;
            let menu = MenuBuilder::new(app).items(&[&toggle, &quit]).build()?;

            let initial_tray_icon = {
                let resource_dir = app.path().resource_dir()
                    .map_err(|e| format!("Failed to get resource dir: {}", e))?;
                let icon_path = resource_dir.join("icons").join("tray-pending.png");
                tauri::image::Image::from_path(&icon_path)
                    .unwrap_or_else(|_| app.default_window_icon().unwrap().clone())
            };

            let _tray = TrayIconBuilder::with_id("main-tray")
                .menu(&menu)
                .show_menu_on_left_click(false)
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
                .icon(initial_tray_icon)
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

            // Start background pipeline poller
            start_background_poller(app.handle().clone());

            info!("Application setup complete");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application")
}
