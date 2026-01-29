import { invoke } from "@tauri-apps/api/core";
import { triggerManualRefresh } from "./pipelines.svelte";

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

interface RefsCache {
    refs: string[];
    timestamp: number;
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

// Cache for pipeline references (branches/tags)
// Key format: "serverName:projectPath"
export const refsCache = $state<Record<string, RefsCache>>({});

// State for tracking loading of refs
export const refsCacheState = $state({
    loading: new Set<string>(),
    errors: {} as Record<string, string>
});

// Helper to generate cache key
function getRefsCacheKey(ciServerName: string, projectPath: string): string {
    return `${ciServerName}:${projectPath}`;
}

// Get cached refs for a project (returns immediately with cached data if available)
export function getCachedRefs(ciServerName: string, projectPath: string): string[] {
    const key = getRefsCacheKey(ciServerName, projectPath);
    return refsCache[key]?.refs || [];
}

// Check if refs are currently loading
export function isRefsLoading(ciServerName: string, projectPath: string): boolean {
    const key = getRefsCacheKey(ciServerName, projectPath);
    return refsCacheState.loading.has(key);
}

// Get error for refs loading
export function getRefsError(ciServerName: string, projectPath: string): string | undefined {
    const key = getRefsCacheKey(ciServerName, projectPath);
    return refsCacheState.errors[key];
}

// Load refs for a project (with caching)
// Returns cached data immediately and refreshes in background
export async function loadRefs(
    ciServerName: string,
    projectPath: string,
    forceRefresh: boolean = false
): Promise<string[]> {
    const key = getRefsCacheKey(ciServerName, projectPath);
    const cached = refsCache[key];

    // If already loading, return cached or empty
    if (refsCacheState.loading.has(key)) {
        return cached?.refs || [];
    }

    // If we have cache and not forcing refresh, return it
    // But still trigger a background refresh
    if (cached && !forceRefresh) {
        // Trigger background refresh (don't await)
        refreshRefsInBackground(ciServerName, projectPath);
        return cached.refs;
    }

    // No cache or force refresh - load synchronously
    return await fetchRefs(ciServerName, projectPath);
}

// Fetch refs from server (internal)
async function fetchRefs(ciServerName: string, projectPath: string): Promise<string[]> {
    const key = getRefsCacheKey(ciServerName, projectPath);

    // Mark as loading
    refsCacheState.loading.add(key);
    delete refsCacheState.errors[key];

    try {
        const refs = await invoke<string[]>("get_pipeline_references", {
            ciServerName,
            projectName: projectPath
        });

        // Update cache
        refsCache[key] = {
            refs,
            timestamp: Date.now()
        };

        return refs;
    } catch (error) {
        console.error(`Failed to load refs for ${key}:`, error);
        refsCacheState.errors[key] = String(error);
        return refsCache[key]?.refs || [];
    } finally {
        refsCacheState.loading.delete(key);
    }
}

// Refresh refs in background (non-blocking)
export function refreshRefsInBackground(ciServerName: string, projectPath: string): void {
    const key = getRefsCacheKey(ciServerName, projectPath);

    // Don't refresh if already loading
    if (refsCacheState.loading.has(key)) {
        return;
    }

    // Fire and forget
    fetchRefs(ciServerName, projectPath).catch(console.error);
}

// Clear refs cache for a specific project
export function clearRefsCache(ciServerName: string, projectPath: string): void {
    const key = getRefsCacheKey(ciServerName, projectPath);
    delete refsCache[key];
    delete refsCacheState.errors[key];
}

// Clear all refs cache
export function clearAllRefsCache(): void {
    for (const key of Object.keys(refsCache)) {
        delete refsCache[key];
    }
    for (const key of Object.keys(refsCacheState.errors)) {
        delete refsCacheState.errors[key];
    }
}

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

    // Trigger immediate pipeline fetch for the new watcher
    const newWatcher = watchersState.watchers.find(
        w => w.name === name && w.ci_server_name === ciServerName && w.project_path === projectPath
    );
    if (newWatcher) {
        triggerManualRefresh(newWatcher.id);
    }
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
