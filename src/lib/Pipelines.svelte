<script lang="ts">
    import Pipeline from "./Pipeline.svelte";
    import {invoke} from "@tauri-apps/api/core";
    import { Toast, Button, Label, Select } from 'flowbite-svelte';
    import { onMount, onDestroy } from 'svelte';

    interface PipelineInterface {
        ref: string;
        web_url: string;
        id: number;
        status: string;
        created_at: string;
        updated_at: string | null;
        finished_at: string | null;
    }

    interface CiServer {
        name: string;
        server_type: string;
        url_string: string;
        api_key: string;
    }

    let pipes: PipelineInterface[] = [];
    let servers: CiServer[] = [];
    let selectedServer = "";
    let projectName = ""; // User needs to enter this
    let selectedReference = ""; // Will be populated from API
    let references: string[] = [];
    let loading = false;
    let error = "";

    let intervalId: number | null = null;
    let ms = 10000;
    let loadReferencesTimeout: number | null = null;

    async function loadServers() {
        try {
            const data = await invoke<CiServer[]>("read_ci_servers");
            servers = data;
            if (servers.length > 0 && !selectedServer) {
                selectedServer = servers[0].name;
            }
        } catch (err) {
            console.error("Failed to load CI servers:", err);
            error = `Failed to load CI servers: ${err}`;
            servers = [];
        }
    }

    async function loadReferences() {
        if (!selectedServer || !projectName) {
            references = [];
            selectedReference = "";
            return;
        }

        try {
            const refs = await invoke('get_pipeline_references', {
                ciServerName: selectedServer,
                projectName: projectName
            }) as string[];
            references = refs;
            if (refs.length > 0 && !selectedReference) {
                selectedReference = refs[0];
            }
        } catch (err) {
            console.error("Failed to load references:", err);
            error = `Failed to load references: ${err}`;
            references = [];
            selectedReference = "";
        }
    }

    function debouncedLoadReferences() {
        if (loadReferencesTimeout) {
            clearTimeout(loadReferencesTimeout);
        }
        loadReferencesTimeout = setTimeout(loadReferences, 500);
    }

    async function get_pipelines() {
        if (!selectedServer || !projectName || !selectedReference) {
            return;
        }

        try {
            loading = true;
            error = "";
            const data = await invoke('get_pipelines', {
                ciServerName: selectedServer,
                projectName: projectName,
                reference: selectedReference
            }) as PipelineInterface[];
            pipes = data;
        } catch (err) {
            console.error("Failed to get pipelines:", err);
            error = `Failed to get pipelines: ${err}`;
            pipes = [];
        } finally {
            loading = false;
        }
    }

    function startPolling() {
        if (intervalId) clearInterval(intervalId);
        intervalId = setInterval(get_pipelines, ms);
    }

    function stopPolling() {
        if (intervalId) {
            clearInterval(intervalId);
            intervalId = null;
        }
    }

    // Load references when server or project changes (debounced)
    $: if (selectedServer && projectName) {
        debouncedLoadReferences();
    }

    // Auto-fetch pipelines when all required fields are set
    $: if (selectedServer && projectName && selectedReference) {
        get_pipelines();
        startPolling();
    } else {
        stopPolling();
    }

    onMount(() => {
        loadServers();
    });

    onDestroy(() => {
        stopPolling();
    });
</script>

<div class="flex justify-center">
    <div class="container mx-auto flex flex-col px-4 drop-shadow-lg">
        <!-- Configuration Section -->
        <div class="bg-white rounded-lg p-6 mb-6 shadow-sm">
            <h3 class="text-lg font-semibold mb-4">Pipeline Configuration</h3>
            <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                <div>
                    <Label for="server-select" class="block mb-2">CI Server</Label>
                    <Select id="server-select" bind:value={selectedServer}>
                        {#each servers as server}
                            <option value={server.name}>{server.name} ({server.server_type})</option>
                        {/each}
                    </Select>
                </div>
                <div>
                    <Label for="project-input" class="block mb-2">Project Path</Label>
                    <input 
                        id="project-input"
                        type="text" 
                        bind:value={projectName}
                        placeholder="owner/project-name"
                        class="bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block w-full p-2.5"
                    />
                </div>
                <div>
                    <Label for="ref-select" class="block mb-2">Branch/Tag</Label>
                    <Select id="ref-select" bind:value={selectedReference}>
                        {#each references as ref}
                            <option value={ref}>{ref}</option>
                        {/each}
                    </Select>
                </div>
            </div>
            <div class="mt-4">
                <Button
                    on:click={get_pipelines}
                    disabled={loading || !selectedServer || !projectName || !selectedReference}
                >
                    {loading ? 'Loading...' : 'Refresh Pipelines'}
                </Button>
            </div>
        </div>

        <!-- Error Display -->
        {#if error}
            <Toast color="red" class="mb-4">
                <svelte:fragment slot="icon">
                    <span class="sr-only">Error icon</span>
                </svelte:fragment>
                {error}
            </Toast>
        {/if}

        <!-- No Servers Message -->
        {#if servers.length === 0}
            <div class="flex justify-center">
                <p class="text-gray-500">No CI servers configured. Please add a CI server first.</p>
            </div>
        {:else if loading}
            <div class="flex justify-center">
                <p>Loading pipelines...</p>
            </div>
        {:else if pipes.length === 0 && !error}
            <div class="flex justify-center">
                <p class="text-gray-500">No pipelines found. Check your configuration and try again.</p>
            </div>
        {:else}
            <!-- Pipelines List -->
            {#each pipes as item}
                <div class="flex items-center">
                    <Pipeline name={item.ref} state={item.status} lastExecuted={item.created_at} webUrl={item.web_url}/>
                </div>
            {/each}
        {/if}
    </div>
</div>
