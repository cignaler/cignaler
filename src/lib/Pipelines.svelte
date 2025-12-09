<script lang="ts">
    import Pipeline from "./Pipeline.svelte";
    import { invoke } from "@tauri-apps/api/core";
    import { watchersState, loadWatchers } from "./stores/watchers.svelte";
    import { fly } from "svelte/transition";

    interface PipelineInterface {
        ref: string;
        web_url: string;
        id: number;
        status: string;
        created_at: string;
        updated_at: string | null;
        finished_at: string | null;
    }

    let pipes = $state<PipelineInterface[]>([]);
    let pipelinesLoading = $state(false);
    let pipelinesError = $state("");

    async function loadAllPipelines() {
        if (watchersState.watchers.length === 0) {
            pipes = [];
            return;
        }

        const allPipelines: PipelineInterface[] = [];
        pipelinesLoading = true;
        pipelinesError = "";

        for (const watcher of watchersState.watchers) {
            if (!watcher.enabled || !watcher.default_branch) continue;

            try {
                const pipelineData = await invoke<PipelineInterface[]>("get_pipelines", {
                    ciServerName: watcher.ci_server_name,
                    projectName: watcher.project_path,
                    reference: watcher.default_branch,
                });
                allPipelines.push(...pipelineData);
            } catch (err) {
                console.error(`Failed to load pipelines for ${watcher.name}:`, err);
            }
        }

        pipes = allPipelines;
        pipelinesLoading = false;
        console.log("Loaded pipelines:", pipes);
    }

    // Load watchers on mount
    $effect(() => {
        loadWatchers();
    });

    // Load pipelines whenever watchers change
    $effect(() => {
        if (!watchersState.loading) {
            loadAllPipelines();
        }
    });
</script>

<div class="flex justify-center">
    <div class="container mx-auto flex flex-col px-4">
        {#if watchersState.loading || pipelinesLoading}
            <div class="flex justify-center py-12">
                <p class="text-gray-500">Loading pipeline watchers...</p>
            </div>
        {:else if watchersState.error}
            <div class="flex justify-center py-12">
                <p class="text-red-500">Error loading watchers: {watchersState.error}</p>
            </div>
        {:else if watchersState.watchers.length === 0}
            <div class="flex justify-center py-12">
                <p class="text-gray-500">No pipeline watchers configured. Click the + button to add one.</p>
            </div>
        {:else if pipes.length === 0 && !pipelinesLoading}
            <div class="flex justify-center py-12">
                <p class="text-gray-500">No pipelines found for configured watchers.</p>
            </div>
        {:else}
            <!-- Pipelines List -->
            {#each pipes as item, index (item.id)}
                <div
                    class="flex items-center"
                    in:fly={{ y: 20, duration: 400, delay: index * 60, opacity: 0 }}
                >
                    <Pipeline name={item.ref} state={item.status} lastExecuted={item.created_at} webUrl={item.web_url}/>
                </div>
            {/each}
        {/if}
    </div>
</div>
