use crate::database::database::{
    read_ci_servers_data, read_cached_pipelines, read_projects_data, save_cached_pipelines,
    save_cached_pipelines_error,
};
use crate::gitlab_client::gitlab_client::get_gitlab_pipelines;
use chrono::Utc;
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};
use tokio::time::{interval, Duration};
use tracing::{debug, error, info, warn};

#[derive(Clone, Serialize)]
pub struct PipelineUpdatePayload {
    pub watcher_id: i64,
    pub pipelines: serde_json::Value,
    pub last_updated: String,
    pub error: Option<String>,
}

pub fn start_background_poller(app_handle: AppHandle) {
    tauri::async_runtime::spawn(async move {
        info!("Background pipeline poller started");

        // Let frontend initialize before competing for resources
        tokio::time::sleep(Duration::from_secs(3)).await;
        poll_all_watchers(&app_handle).await;

        let mut ticker = interval(Duration::from_secs(60));
        // The first tick completes immediately, consume it since we already polled
        ticker.tick().await;

        loop {
            ticker.tick().await;
            debug!("Background poller tick");
            poll_all_watchers(&app_handle).await;
        }
    });
}

async fn poll_all_watchers(app_handle: &AppHandle) {
    let projects = match read_projects_data() {
        Ok(p) => p,
        Err(e) => {
            error!("Failed to read projects for polling: {}", e);
            return;
        }
    };

    let servers = match read_ci_servers_data() {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to read CI servers for polling: {}", e);
            return;
        }
    };

    let enabled: Vec<_> = projects.iter().filter(|p| p.enabled && p.default_branch.is_some()).collect();
    if enabled.is_empty() {
        debug!("No enabled watchers with a default branch, skipping poll");
        return;
    }

    info!("Polling {} enabled watchers", enabled.len());

    let mut worst_status: Option<String> = None;

    for project in &enabled {
        let branch = match &project.default_branch {
            Some(b) => b.clone(),
            None => continue,
        };

        let server = match servers.iter().find(|s| s.name == project.ci_server_name) {
            Some(s) => s.clone(),
            None => {
                warn!("CI server '{}' not found for project '{}'", project.ci_server_name, project.name);
                continue;
            }
        };

        let project_path = project.project_path.clone();
        let project_id = project.id;
        let project_name = project.name.clone();

        let result = tokio::task::spawn_blocking(move || {
            get_gitlab_pipelines(&branch, &project_path, &server)
        })
        .await;

        let now = Utc::now().to_rfc3339();

        match result {
            Ok(Ok(pipelines)) => {
                let json = match serde_json::to_string(&pipelines) {
                    Ok(j) => j,
                    Err(e) => {
                        error!("Failed to serialize pipelines for '{}': {}", project_name, e);
                        continue;
                    }
                };

                if let Err(e) = save_cached_pipelines(project_id, &json, &now, None) {
                    error!("Failed to save cached pipelines for '{}': {}", project_name, e);
                }

                // Determine worst status from this watcher's latest pipeline
                if let Some(first) = pipelines.first() {
                    update_worst_status(&mut worst_status, &first.status);
                }

                let payload = PipelineUpdatePayload {
                    watcher_id: project_id,
                    pipelines: serde_json::from_str(&json).unwrap_or(serde_json::Value::Array(vec![])),
                    last_updated: now,
                    error: None,
                };

                if let Err(e) = app_handle.emit("pipeline-update", &payload) {
                    error!("Failed to emit pipeline-update event: {}", e);
                }

                debug!("Polled '{}': {} pipelines", project_name, pipelines.len());
            }
            Ok(Err(e)) => {
                warn!("Pipeline fetch failed for '{}': {}", project_name, e);
                let error_msg = e.to_string();

                if let Err(db_err) = save_cached_pipelines_error(project_id, &now, &error_msg) {
                    error!("Failed to save pipeline error for '{}': {}", project_name, db_err);
                }

                // Read existing cached data to emit with error
                let cached_json = read_cached_pipelines(project_id)
                    .ok()
                    .flatten()
                    .map(|r| r.pipelines_json)
                    .unwrap_or_else(|| "[]".to_string());

                let payload = PipelineUpdatePayload {
                    watcher_id: project_id,
                    pipelines: serde_json::from_str(&cached_json).unwrap_or(serde_json::Value::Array(vec![])),
                    last_updated: now,
                    error: Some(error_msg),
                };

                if let Err(e) = app_handle.emit("pipeline-update", &payload) {
                    error!("Failed to emit pipeline-update event: {}", e);
                }
            }
            Err(e) => {
                error!("Spawn_blocking panicked for '{}': {}", project_name, e);
            }
        }
    }

    // Update tray icon based on worst status across all watchers
    let state = worst_status.as_deref().unwrap_or("pending");
    if let Err(e) = set_tray_icon(app_handle, state) {
        error!("Failed to update tray icon: {}", e);
    }
}

pub async fn poll_single_watcher(app_handle: &AppHandle, watcher_id: i64) {
    let projects = match read_projects_data() {
        Ok(p) => p,
        Err(e) => {
            error!("Failed to read projects: {}", e);
            return;
        }
    };

    let project = match projects.iter().find(|p| p.id == watcher_id) {
        Some(p) => p,
        None => {
            warn!("Watcher id={} not found", watcher_id);
            return;
        }
    };

    let branch = match &project.default_branch {
        Some(b) => b.clone(),
        None => {
            warn!("Watcher '{}' has no default branch", project.name);
            return;
        }
    };

    let servers = match read_ci_servers_data() {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to read CI servers: {}", e);
            return;
        }
    };

    let server = match servers.iter().find(|s| s.name == project.ci_server_name) {
        Some(s) => s.clone(),
        None => {
            warn!("CI server '{}' not found", project.ci_server_name);
            return;
        }
    };

    let project_path = project.project_path.clone();
    let project_id = project.id;
    let project_name = project.name.clone();

    let result = tokio::task::spawn_blocking(move || {
        get_gitlab_pipelines(&branch, &project_path, &server)
    })
    .await;

    let now = Utc::now().to_rfc3339();

    match result {
        Ok(Ok(pipelines)) => {
            let json = match serde_json::to_string(&pipelines) {
                Ok(j) => j,
                Err(e) => {
                    error!("Failed to serialize pipelines for '{}': {}", project_name, e);
                    return;
                }
            };

            if let Err(e) = save_cached_pipelines(project_id, &json, &now, None) {
                error!("Failed to save cached pipelines for '{}': {}", project_name, e);
            }

            let payload = PipelineUpdatePayload {
                watcher_id: project_id,
                pipelines: serde_json::from_str(&json).unwrap_or(serde_json::Value::Array(vec![])),
                last_updated: now,
                error: None,
            };

            if let Err(e) = app_handle.emit("pipeline-update", &payload) {
                error!("Failed to emit pipeline-update event: {}", e);
            }

            info!("Manual refresh for '{}': {} pipelines", project_name, pipelines.len());
        }
        Ok(Err(e)) => {
            warn!("Manual refresh failed for '{}': {}", project_name, e);
            let error_msg = e.to_string();

            if let Err(db_err) = save_cached_pipelines_error(project_id, &now, &error_msg) {
                error!("Failed to save pipeline error for '{}': {}", project_name, db_err);
            }

            let cached_json = read_cached_pipelines(project_id)
                .ok()
                .flatten()
                .map(|r| r.pipelines_json)
                .unwrap_or_else(|| "[]".to_string());

            let payload = PipelineUpdatePayload {
                watcher_id: project_id,
                pipelines: serde_json::from_str(&cached_json).unwrap_or(serde_json::Value::Array(vec![])),
                last_updated: now,
                error: Some(error_msg),
            };

            if let Err(e) = app_handle.emit("pipeline-update", &payload) {
                error!("Failed to emit pipeline-update event: {}", e);
            }
        }
        Err(e) => {
            error!("Spawn_blocking panicked for '{}': {}", project_name, e);
        }
    }

    // After single watcher poll, update tray based on all cached data
    update_tray_from_all_cached(app_handle);
}

fn update_worst_status(current: &mut Option<String>, status: &str) {
    let priority = match status {
        "failed" => 2,
        "running" | "pending" | "created" | "waiting_for_resource" | "preparing" | "scheduled" => 1,
        "success" => 0,
        _ => 1, // unknown statuses treated as pending
    };

    let current_priority = current.as_deref().map(|s| match s {
        "failed" => 2,
        "running" | "pending" | "created" | "waiting_for_resource" | "preparing" | "scheduled" => 1,
        "success" => 0,
        _ => 1,
    }).unwrap_or(-1_i32) as i32;

    if priority as i32 > current_priority {
        *current = Some(status_to_tray_state(status).to_string());
    }
}

fn status_to_tray_state(status: &str) -> &str {
    match status {
        "success" => "success",
        "failed" => "failed",
        _ => "pending",
    }
}

fn update_tray_from_all_cached(app_handle: &AppHandle) {
    let projects = match read_projects_data() {
        Ok(p) => p,
        Err(_) => return,
    };

    let mut worst_status: Option<String> = None;

    for project in projects.iter().filter(|p| p.enabled) {
        if let Ok(Some(cached)) = read_cached_pipelines(project.id) {
            if let Ok(pipelines) = serde_json::from_str::<Vec<serde_json::Value>>(&cached.pipelines_json) {
                if let Some(first) = pipelines.first() {
                    if let Some(status) = first.get("status").and_then(|s| s.as_str()) {
                        update_worst_status(&mut worst_status, status);
                    }
                }
            }
        }
    }

    let state = worst_status.as_deref().unwrap_or("pending");
    if let Err(e) = set_tray_icon(app_handle, state) {
        error!("Failed to update tray icon: {}", e);
    }
}

pub fn set_tray_icon(app_handle: &AppHandle, state: &str) -> Result<(), String> {
    let icon_filename = match state {
        "success" => "tray-success.png",
        "failed" => "tray-failed.png",
        _ => "tray-pending.png",
    };

    let resource_dir = app_handle
        .path()
        .resource_dir()
        .map_err(|e| format!("Failed to get resource dir: {}", e))?;

    let icon_path = resource_dir.join("icons").join(icon_filename);
    debug!("Loading tray icon from: {:?}", icon_path);

    let icon = tauri::image::Image::from_path(&icon_path)
        .map_err(|e| format!("Failed to load icon from {:?}: {}", icon_path, e))?;

    if let Some(tray) = app_handle.tray_by_id("main-tray") {
        tray.set_icon(Some(icon))
            .map_err(|e| format!("Failed to set tray icon: {}", e))?;
        debug!("Tray icon updated to: {}", state);
        Ok(())
    } else {
        Err("Tray icon with ID 'main-tray' not found".to_string())
    }
}
