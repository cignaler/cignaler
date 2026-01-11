import { invoke } from "@tauri-apps/api/core";

export interface ProjectWatcher {
    id: number;
    name: string;
    ci_server_name: string;
    project_path: string;
    default_branch: string | null;
    enabled: boolean;
}

interface CiServer {
    name: string;
    server_type: string;
    url_string: string;
    api_key: string;
}

// Shared reactive state
export const watchersState = $state({
    watchers: [] as ProjectWatcher[],
    loading: true,  // Start as true to show loading on first render
    error: null as string | null
});

export const serversState = $state({
    servers: [] as CiServer[],
    loading: false,
    error: null as string | null
});

// Load watchers from database
export async function loadWatchers() {
    try {
        watchersState.loading = true;
        watchersState.error = null;
        const projects = await invoke<ProjectWatcher[]>("read_projects");
        watchersState.watchers = projects;
        console.log("Loaded watchers:", projects.length);
    } catch (err) {
        console.error("Failed to load watchers:", err);
        watchersState.error = String(err);
        watchersState.watchers = [];
    } finally {
        watchersState.loading = false;
    }
}

// Load CI servers from database
export async function loadServers() {
    try {
        serversState.loading = true;
        serversState.error = null;
        const data = await invoke<CiServer[]>("read_ci_servers");
        serversState.servers = data;
        console.log("Loaded CI servers:", data.length);
    } catch (err) {
        console.error("Failed to load CI servers:", err);
        serversState.error = String(err);
        serversState.servers = [];
    } finally {
        serversState.loading = false;
    }
}

// Add a new watcher and update state
export async function addWatcher(
    name: string,
    ciServerName: string,
    projectPath: string,
    defaultBranch: string
) {
    await invoke("store_project_data", {
        name,
        ciServerName,
        projectPath,
        defaultBranch
    });

    // Reload watchers to get the updated list
    await loadWatchers();
}

// Add a new CI server and update state
export async function addServer(
    name: string,
    serverType: string,
    urlString: string,
    apiKey: string
) {
    await invoke("store_ci_server_data", {
        name,
        serverType,
        urlString,
        apiKey
    });

    // Reload servers to get the updated list
    await loadServers();
}

// Update a CI server and refresh state
export async function updateServer(
    name: string,
    serverType: string,
    urlString: string,
    apiKey: string
) {
    await invoke("update_ci_server", {
        name,
        serverType,
        urlString,
        apiKey
    });

    // Reload servers to get the updated list
    await loadServers();
}

// Delete a CI server and refresh state
export async function deleteServer(name: string) {
    await invoke("delete_ci_server", { name });

    // Reload servers to get the updated list
    await loadServers();
}

// Update a watcher and refresh state
export async function updateWatcher(
    id: number,
    name: string,
    ciServerName: string,
    projectPath: string,
    defaultBranch: string
) {
    await invoke("update_project", {
        id,
        name,
        ciServerName,
        projectPath,
        defaultBranch
    });

    // Reload watchers to get the updated list
    await loadWatchers();
}

// Delete a watcher and refresh state
export async function deleteWatcher(id: number) {
    await invoke("delete_project", { id });

    // Reload watchers to get the updated list
    await loadWatchers();
}

// Toggle watcher enabled state
export async function toggleWatcherEnabled(id: number, enabled: boolean) {
    await invoke("set_project_enabled", { id, enabled });

    // Reload watchers to get the updated list
    await loadWatchers();
}
