<script lang="ts">
  import Toast from "./components/ui/Toast.svelte";
  import Config from "./Config.svelte";
  import { serversState, loadServers } from "./stores/watchers.svelte";
  import { fly } from "svelte/transition";

  let {
    onedit
  }: {
    onedit?: (detail: { name: string; urlString: string; secret: string; serverType: string }) => void;
  } = $props();

  // Load servers on mount
  $effect(() => {
    loadServers();
  });
</script>

<div>
  {#if serversState.loading}
    <div class="flex justify-center">
      <p>Loading...</p>
    </div>
  {:else if serversState.error}
    <Toast color="red" class="flex items-center">
      {#snippet icon()}
        <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 20 20" aria-hidden="true">
          <path fill-rule="evenodd" d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z" clip-rule="evenodd"/>
        </svg>
        <span class="sr-only">Warning icon</span>
      {/snippet}
      {serversState.error}
    </Toast>
  {:else if serversState.servers.length === 0}
    <div class="flex justify-center">
      <p class="text-gray-500">No CI servers configured. Click the + button to add one.</p>
    </div>
  {:else}
    {#each serversState.servers as server, index (server.name)}
      <div
        class="flex items-center"
        in:fly={{ y: 20, duration: 400, delay: index * 60, opacity: 0 }}
      >
        <Config
          name={server.name}
          urlString={server.url_string}
          secret={server.api_key}
          serverType={server.server_type}
          {onedit}
        />
      </div>
    {/each}
  {/if}
</div>
