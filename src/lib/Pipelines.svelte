<script lang="ts">
    import Pipeline from "./Pipeline.svelte";
    import { invoke } from "@tauri-apps/api/core";
    import { watchersState, loadWatchers } from "./stores/watchers.svelte";
    import { onDestroy } from "svelte";
    import { fade } from "svelte/transition";

    interface Props {
        selectedWatcherId: number | null;
        onpreferences: () => void;
    }

    let { selectedWatcherId, onpreferences }: Props = $props();

    interface PipelineInterface {
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

    let pipes = $state<PipelineInterface[]>([]);
    let pipelinesLoading = $state(false);
    let pipelinesError = $state("");
    let lastSync = $state<Date>(new Date());
    let refreshInterval: ReturnType<typeof setInterval> | null = null;

    const REFRESH_INTERVAL_MS = 30000; // 30 seconds

    let selectedWatcher = $derived(
        watchersState.watchers.find(w => w.id === selectedWatcherId)
    );

    // Update tray icon based on pipeline state
    async function updateTrayIcon(state: string) {
        try {
            await invoke("update_tray_icon", { state });
        } catch (err) {
            console.error("Failed to update tray icon:", err);
        }
    }

    async function loadPipelines() {
        if (!selectedWatcher || !selectedWatcher.enabled || !selectedWatcher.default_branch) {
            pipes = [];
            return;
        }

        pipelinesLoading = true;
        pipelinesError = "";

        try {
            const pipelineData = await invoke<PipelineInterface[]>("get_pipelines", {
                ciServerName: selectedWatcher.ci_server_name,
                projectName: selectedWatcher.project_path,
                reference: selectedWatcher.default_branch,
            });
            pipes = pipelineData;
            lastSync = new Date();
        } catch (err) {
            console.error(`Failed to load pipelines for ${selectedWatcher.name}:`, err);
            pipelinesError = String(err);
        } finally {
            pipelinesLoading = false;
        }
    }

    function formatLastSync(): string {
        const now = new Date();
        const diff = now.getTime() - lastSync.getTime();
        const totalSeconds = Math.floor(diff / 1000);
        const hours = Math.floor(totalSeconds / 3600);
        const minutes = Math.floor((totalSeconds % 3600) / 60);
        const seconds = totalSeconds % 60;

        return `${String(hours).padStart(2, '0')}:${String(minutes).padStart(2, '0')}:${String(seconds).padStart(2, '0')}`;
    }

    // Load watchers on mount
    $effect(() => {
        loadWatchers();
    });

    // Load pipelines when selected watcher changes & set up auto-refresh
    $effect(() => {
        // Clear existing interval
        if (refreshInterval) {
            clearInterval(refreshInterval);
            refreshInterval = null;
        }

        if (selectedWatcher) {
            // Initial load
            loadPipelines();

            // Set up auto-refresh every 30 seconds
            refreshInterval = setInterval(loadPipelines, REFRESH_INTERVAL_MS);
        }

        // Cleanup on effect re-run or unmount
        return () => {
            if (refreshInterval) {
                clearInterval(refreshInterval);
                refreshInterval = null;
            }
        };
    });

    // Also clean up on component destroy
    onDestroy(() => {
        if (refreshInterval) {
            clearInterval(refreshInterval);
        }
    });

    // Update tray icon when pipelines change
    $effect(() => {
        if (pipes.length > 0) {
            // Find the most recent pipeline by sorting by updated_at or created_at
            const sortedPipes = [...pipes].sort((a, b) => {
                const dateA = new Date(a.updated_at || a.created_at).getTime();
                const dateB = new Date(b.updated_at || b.created_at).getTime();
                return dateB - dateA; // Most recent first
            });

            // Use the state of the most recent pipeline
            const lastPipeline = sortedPipes[0];
            if (lastPipeline) {
                updateTrayIcon(lastPipeline.status);
            }
        } else if (!pipelinesLoading && !watchersState.loading) {
            // No pipelines found, set to pending state
            updateTrayIcon("pending");
        }
    });
</script>

<!-- Header with selected watcher info -->
<div class="bg-white border-b border-gray-200 px-8 py-6">
    <div class="flex items-start justify-between">
        <div class="flex-1">
            {#if selectedWatcher}
                <div class="flex items-center gap-3 mb-2">
                    <h1 class="text-2xl font-bold text-gray-900">{selectedWatcher.name}</h1>
                    {#if selectedWatcher.enabled}
                        <span class="bg-green-100 text-green-700 text-xs font-medium px-2.5 py-1 rounded">Active</span>
                    {:else}
                        <span class="bg-gray-100 text-gray-700 text-xs font-medium px-2.5 py-1 rounded">Idle</span>
                    {/if}
                </div>
                <p class="text-sm text-gray-500">
                    {selectedWatcher.project_path} @ {selectedWatcher.default_branch || 'default'}
                </p>
            {:else}
                <h1 class="text-2xl font-bold text-gray-900">No Watcher Selected</h1>
                <p class="text-sm text-gray-500">Select a watcher from the sidebar to view pipelines.</p>
            {/if}
        </div>
        <div class="flex items-center gap-4">
            <div class="flex items-center gap-2 text-sm text-gray-500">
                <span>Last sync: {formatLastSync()}</span>
                <button
                    onclick={loadPipelines}
                    class="p-1.5 hover:bg-gray-100 rounded-lg transition-colors"
                    disabled={pipelinesLoading}
                    aria-label="Refresh pipelines"
                >
                    <svg class="w-4 h-4 {pipelinesLoading ? 'animate-spin' : ''}" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"/>
                    </svg>
                </button>
            </div>
            <button
                onclick={onpreferences}
                class="flex items-center gap-2 px-3 py-2 text-sm text-gray-700 hover:bg-gray-100 rounded-lg transition-colors"
            >
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"/>
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"/>
                </svg>
                Preferences
            </button>
        </div>
    </div>
</div>

<!-- Pipelines List -->
<div class="flex-1 overflow-y-auto bg-gray-50">
    {#if watchersState.loading || pipelinesLoading}
        <div class="flex justify-center py-12">
            <p class="text-sm text-gray-500">Loading pipelines...</p>
        </div>
    {:else if !selectedWatcher}
        <div class="flex justify-center py-12" transition:fade={{ duration: 200 }}>
            <p class="text-sm text-gray-500">Select a watcher to view pipelines</p>
        </div>
    {:else if pipelinesError}
        <div class="flex justify-center py-12" transition:fade={{ duration: 200 }}>
            <p class="text-sm text-red-500">Error: {pipelinesError}</p>
        </div>
    {:else if pipes.length === 0}
        <div class="flex flex-col items-center justify-center py-12" transition:fade={{ duration: 200 }}>
            <div class="animate-pulse mb-4">
                <svg class="w-16 h-16 text-gray-300" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2"/>
                </svg>
            </div>
            <p class="text-sm text-gray-500">No pipelines found</p>
        </div>
    {:else}
        <div class="bg-white">
            {#each pipes as item (item.id)}
                <Pipeline
                    name={item.ref}
                    state={item.status}
                    lastExecuted={item.created_at}
                    finishedAt={item.finished_at}
                    sha={item.sha}
                    source={item.source}
                    webUrl={item.web_url}
                />
            {/each}
        </div>
        <div class="flex justify-center py-8">
            <button class="flex items-center gap-2 text-sm text-gray-600 hover:text-orange-600 transition-colors">
                <span>View archived pipelines</span>
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 8l4 4m0 0l-4 4m4-4H3"/>
                </svg>
            </button>
        </div>
    {/if}
</div>
