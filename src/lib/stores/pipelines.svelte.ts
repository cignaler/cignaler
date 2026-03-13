import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export interface PipelineEntry {
    ref: string;
    web_url: string;
    id: number;
    status: string;
    created_at: string;
    updated_at: string | null;
    finished_at: string | null;
    sha: string | null;
    source: string | null;
}

export interface WatcherPipelineEntry {
    pipelines: PipelineEntry[];
    lastUpdated: string | null;
    error: string | null;
}

interface PipelineUpdateEvent {
    watcher_id: number;
    pipelines: PipelineEntry[];
    last_updated: string;
    error: string | null;
}

interface CachedPipelinesResponse {
    watcher_id: number;
    pipelines: PipelineEntry[];
    last_updated: string | null;
    error: string | null;
}

// Reactive cache of pipeline data per watcher id
export const pipelinesCache = $state<Record<number, WatcherPipelineEntry>>({});

let unlisten: UnlistenFn | null = null;

export async function initPipelineListener(): Promise<void> {
    // Prevent double-registering
    if (unlisten) return;

    unlisten = await listen<PipelineUpdateEvent>("pipeline-update", (event) => {
        const { watcher_id, pipelines, last_updated, error } = event.payload;
        pipelinesCache[watcher_id] = {
            pipelines,
            lastUpdated: last_updated,
            error,
        };
    });
}

export async function fetchCachedPipelines(watcherId: number): Promise<void> {
    // If we already have data in the reactive cache, skip the DB round-trip
    if (pipelinesCache[watcherId]?.pipelines?.length) return;

    try {
        const response = await invoke<CachedPipelinesResponse>("get_cached_pipelines", {
            watcherId,
        });
        pipelinesCache[watcherId] = {
            pipelines: response.pipelines,
            lastUpdated: response.last_updated,
            error: response.error,
        };
    } catch (err) {
        console.error(`Failed to fetch cached pipelines for watcher ${watcherId}:`, err);
    }
}

export async function triggerManualRefresh(watcherId: number): Promise<void> {
    try {
        await invoke("trigger_poll", { watcherId });
    } catch (err) {
        console.error(`Failed to trigger poll for watcher ${watcherId}:`, err);
    }
}
