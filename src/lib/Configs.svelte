<script lang="ts">
  import { Toast } from "flowbite-svelte";
  import { invoke } from "@tauri-apps/api/core";
  import Config from "./Config.svelte";
  import { onMount, createEventDispatcher } from "svelte";

  export let refreshTrigger = 0; // Used to trigger refresh from parent

  const dispatch = createEventDispatcher();

  interface CiServer {
    name: string;
    server_type: string;
    url_string: string;
    api_key: string;
  }

  let servers: CiServer[] = [];
  let loading = true;
  let error = null;

  async function loadServers() {
    try {
      loading = true;
      error = null;
      const data = await invoke<CiServer[]>("read_ci_servers");
      servers = data;
      console.log("Loaded CI servers:", servers.length, "servers");
    } catch (err) {
      console.error("Failed to load CI servers:", err);
      error = err instanceof Error ? err.message : String(err);
      servers = [];
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    loadServers();
  });

  // Watch for refresh trigger changes
  $: if (refreshTrigger > 0) {
    loadServers();
  }
</script>

<div>
  {#if loading}
    <div class="flex justify-center">
      <p>Loading...</p>
    </div>
  {:else if error}
    <Toast color="red" class="flex items-center">
      <svelte:fragment slot="icon">
        <span class="sr-only">Warning icon</span>
      </svelte:fragment>
      {error}
    </Toast>
  {:else if servers.length === 0}
    <div class="flex justify-center">
      <p class="text-gray-500">No CI servers configured. Click the + button to add one.</p>
    </div>
  {:else}
    {#each servers as server}
      <div class="flex items-center">
        <Config
          name={server.name}
          urlString={server.url_string}
          secret={server.api_key}
          serverType={server.server_type}
          on:edit={(e) => dispatch('edit', e.detail)}
        />
      </div>
    {/each}
  {/if}
</div>
