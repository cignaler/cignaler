use crate::database::database::{
    read_cached_pipelines, read_ci_servers_data, read_projects_data, save_cached_pipelines,
    save_cached_pipelines_error,
};
use crate::gitlab_client::gitlab_client::get_gitlab_pipelines;
use crate::CiServer;
use chrono::Utc;
use serde::Serialize;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::Semaphore;
use tokio::task::JoinSet;
use tokio::time::{interval, Duration, MissedTickBehavior};
use tracing::{debug, error, info, warn};

/// Upper bound on watchers polled at the same time, so one slow or
/// rate-limited server can't stall the rest but we also don't hammer
/// an instance hosting many watched projects.
const MAX_CONCURRENT_POLLS: usize = 4;

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
        // If a poll cycle overruns the interval, delay subsequent ticks
        // instead of firing them back-to-back
        ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);
        // The first tick completes immediately, consume it since we already polled
        ticker.tick().await;

        loop {
            ticker.tick().await;
            debug!("Background poller tick");
            poll_all_watchers(&app_handle).await;
        }
    });
}

/// Everything needed to poll one watcher, detached from the DB row.
struct PollTarget {
    project_id: i64,
    project_name: String,
    project_path: String,
    branch: String,
    server: CiServer,
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

    let targets: Vec<PollTarget> = projects
        .iter()
        .filter(|p| p.enabled)
        .filter_map(|p| {
            let branch = p.default_branch.clone()?;
            match servers.iter().find(|s| s.name == p.ci_server_name) {
                Some(server) => Some(PollTarget {
                    project_id: p.id,
                    project_name: p.name.clone(),
                    project_path: p.project_path.clone(),
                    branch,
                    server: server.clone(),
                }),
                None => {
                    warn!(
                        "CI server '{}' not found for project '{}'",
                        p.ci_server_name, p.name
                    );
                    None
                }
            }
        })
        .collect();

    if targets.is_empty() {
        debug!("No enabled watchers with a default branch, skipping poll");
        return;
    }

    info!("Polling {} enabled watchers", targets.len());

    let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT_POLLS));
    let mut tasks = JoinSet::new();

    for target in targets {
        let app = app_handle.clone();
        let semaphore = Arc::clone(&semaphore);
        tasks.spawn(async move {
            let _permit = semaphore
                .acquire_owned()
                .await
                .expect("poll semaphore closed");
            poll_target(&app, target).await
        });
    }

    let mut worst_status: Option<String> = None;
    while let Some(result) = tasks.join_next().await {
        match result {
            Ok(Some(status)) => update_worst_status(&mut worst_status, &status),
            Ok(None) => {}
            Err(e) => error!("Poll task panicked: {}", e),
        }
    }

    // Update tray icon based on worst status across all watchers
    let state = worst_status
        .as_deref()
        .map(status_to_tray_state)
        .unwrap_or("pending");
    if let Err(e) = set_tray_icon(app_handle, state) {
        error!("Failed to update tray icon: {}", e);
    }
}

/// Poll one watcher: fetch pipelines, persist the result, and notify the UI.
/// Returns the latest pipeline's status when the fetch succeeded.
async fn poll_target(app_handle: &AppHandle, target: PollTarget) -> Option<String> {
    let PollTarget {
        project_id,
        project_name,
        project_path,
        branch,
        server,
    } = target;

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
                    return None;
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

            debug!("Polled '{}': {} pipelines", project_name, pipelines.len());
            pipelines.first().map(|p| p.status.clone())
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
                pipelines: serde_json::from_str(&cached_json)
                    .unwrap_or(serde_json::Value::Array(vec![])),
                last_updated: now,
                error: Some(error_msg),
            };

            if let Err(e) = app_handle.emit("pipeline-update", &payload) {
                error!("Failed to emit pipeline-update event: {}", e);
            }

            None
        }
        Err(e) => {
            error!("Pipeline fetch task panicked for '{}': {}", project_name, e);
            None
        }
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

    let target = PollTarget {
        project_id: project.id,
        project_name: project.name.clone(),
        project_path: project.project_path.clone(),
        branch,
        server,
    };

    poll_target(app_handle, target).await;

    // After single watcher poll, update tray based on all cached data
    update_tray_from_all_cached(app_handle);
}

/// Ranks a GitLab pipeline status for tray purposes: failed beats
/// in-progress/unknown, which beats success.
fn status_priority(status: &str) -> u8 {
    match status {
        "failed" => 2,
        "success" => 0,
        // running, pending, created, etc. — and unknown statuses
        _ => 1,
    }
}

fn update_worst_status(current: &mut Option<String>, status: &str) {
    let worse = match current.as_deref() {
        None => true,
        Some(c) => status_priority(status) > status_priority(c),
    };
    if worse {
        *current = Some(status.to_string());
    }
}

fn status_to_tray_state(status: &str) -> &str {
    match status {
        "success" => "success",
        "failed" => "failed",
        _ => "pending",
    }
}

pub fn update_tray_from_all_cached(app_handle: &AppHandle) {
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

    let state = worst_status
        .as_deref()
        .map(status_to_tray_state)
        .unwrap_or("pending");
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn failed_beats_running_beats_success() {
        let mut worst = None;
        update_worst_status(&mut worst, "success");
        assert_eq!(worst.as_deref(), Some("success"));

        update_worst_status(&mut worst, "running");
        assert_eq!(worst.as_deref(), Some("running"));

        // Another success doesn't downgrade
        update_worst_status(&mut worst, "success");
        assert_eq!(worst.as_deref(), Some("running"));

        update_worst_status(&mut worst, "failed");
        assert_eq!(worst.as_deref(), Some("failed"));

        // Nothing beats failed
        update_worst_status(&mut worst, "running");
        assert_eq!(worst.as_deref(), Some("failed"));
    }

    #[test]
    fn unknown_statuses_count_as_in_progress() {
        let mut worst = Some("success".to_string());
        update_worst_status(&mut worst, "some_future_status");
        assert_eq!(worst.as_deref(), Some("some_future_status"));
        assert_eq!(status_to_tray_state("some_future_status"), "pending");
    }

    #[test]
    fn tray_state_mapping() {
        assert_eq!(status_to_tray_state("success"), "success");
        assert_eq!(status_to_tray_state("failed"), "failed");
        assert_eq!(status_to_tray_state("running"), "pending");
        assert_eq!(status_to_tray_state("pending"), "pending");
    }
}
