<script lang="ts">
  import Toast from "./components/ui/Toast.svelte";
  import Config from "./Config.svelte";
  import { serversState, loadServers } from "./stores/watchers.svelte";

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
      <svelte:fragment slot="icon">
        <span class="sr-only">Warning icon</span>
      </svelte:fragment>
      {serversState.error}
    </Toast>
  {:else if serversState.servers.length === 0}
    <div class="flex justify-center">
      <p class="text-gray-500">No CI servers configured. Click the + button to add one.</p>
    </div>
  {:else}
    {#each serversState.servers as server (server.name)}
      <div class="flex items-center">
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
