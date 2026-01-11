<script lang="ts">
  import { watchersState, toggleWatcherEnabled, type ProjectWatcher } from "./stores/watchers.svelte";
  import { getVersion } from "@tauri-apps/api/app";
  import { fade } from "svelte/transition";

  interface Props {
    onadd: () => void;
    onedit: (watcher: ProjectWatcher) => void;
    ondelete: (watcher: ProjectWatcher) => void;
    selectedWatcherId?: number | null;
    onselect?: (id: number) => void;
  }

  let { onadd, onedit, ondelete, selectedWatcherId = null, onselect }: Props = $props();

  let version = $state("...");
  let togglingId = $state<number | null>(null);

  // Load version on mount
  $effect(() => {
    getVersion().then(v => version = v).catch(() => version = "0.0.0");
  });

  async function handleToggleEnabled(watcher: ProjectWatcher, e: Event) {
    e.stopPropagation();
    togglingId = watcher.id;
    try {
      await toggleWatcherEnabled(watcher.id, !watcher.enabled);
    } finally {
      togglingId = null;
    }
  }
</script>

<aside class="
  w-64
  border-r border-gray-200
  bg-white
  flex flex-col
  h-screen
">
  <!-- Logo/Header -->
  <div class="p-5 border-b border-gray-200">
    <div class="flex items-center gap-2">
      <div class="w-8 h-8 bg-orange-500 rounded-lg flex items-center justify-center">
        <svg class="w-5 h-5 text-white" fill="currentColor" viewBox="0 0 20 20">
          <path d="M10 2a6 6 0 00-6 6v3.586l-.707.707A1 1 0 004 14h12a1 1 0 00.707-1.707L16 11.586V8a6 6 0 00-6-6zM10 18a3 3 0 01-3-3h6a3 3 0 01-3 3z"/>
        </svg>
      </div>
      <div>
        <h1 class="text-base font-bold text-gray-900">Cignaler</h1>
        <p class="text-[10px] text-gray-500 uppercase tracking-wide">Pipelines</p>
      </div>
    </div>
  </div>

  <!-- Add Watcher Button -->
  <div class="p-4">
    <button
      onclick={onadd}
      class="
        w-full
        bg-orange-500 hover:bg-orange-600
        text-white
        font-medium text-sm
        py-2.5 px-4
        rounded-lg
        flex items-center justify-center gap-2
        transition-colors
      "
    >
      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4"/>
      </svg>
      <span>Add New Watcher</span>
    </button>
  </div>

  <!-- Watchers Section Header -->
  <div class="px-4 pb-3 flex items-center justify-between">
    <h2 class="text-xs font-semibold text-gray-400 uppercase tracking-wider">Watchers</h2>
    {#if watchersState.watchers.length > 0}
      <span class="text-xs font-medium text-gray-400 bg-gray-100 px-2 py-0.5 rounded-full">
        {watchersState.watchers.length}
      </span>
    {/if}
  </div>

  <!-- Watchers List -->
  <div class="flex-1 overflow-y-auto px-4 pb-4 space-y-1">
    {#if watchersState.loading}
      <div class="text-center py-8">
        <p class="text-xs text-gray-500">Loading watchers...</p>
      </div>
    {:else if watchersState.error}
      <div class="text-center py-8">
        <p class="text-xs text-red-500">Error: {watchersState.error}</p>
      </div>
    {:else if watchersState.watchers.length === 0}
      <div class="text-center py-8" transition:fade={{ duration: 200 }}>
        <div class="animate-pulse mb-3">
          <svg class="w-12 h-12 mx-auto text-gray-300" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"/>
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z"/>
          </svg>
        </div>
        <p class="text-xs text-gray-500">No watchers yet</p>
        <p class="text-[10px] text-gray-400 mt-1">Click the button above to add one</p>
      </div>
    {:else}
      {#each watchersState.watchers as watcher (watcher.id)}
        <div
          class="
            group
            w-full text-left
            p-3 pl-2
            rounded-lg
            border-l-4
            {selectedWatcherId === watcher.id ? 'border-l-orange-500 bg-orange-50/50' : 'border-l-transparent'}
            hover:bg-gray-50
            transition-all
          "
        >
          <div class="flex items-start gap-2.5 pl-1">
            <button
              onclick={() => onselect?.(watcher.id)}
              class="flex items-start gap-2.5 flex-1 min-w-0 text-left"
            >
              <span class="
                w-2 h-2
                rounded-full
                {watcher.enabled ? 'bg-green-500' : 'bg-gray-300'}
                shrink-0
                mt-1.5
              "></span>
              <span class="flex-1 min-w-0">
                <span class="block font-semibold text-sm text-gray-900">
                  {watcher.name}
                </span>
                <span class="block text-xs text-gray-500 mt-0.5">
                  {watcher.enabled ? 'Monitoring' : 'Idle'}
                </span>
              </span>
            </button>
            <div class="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
              <!-- Toggle enabled button -->
              <button
                onclick={(e) => handleToggleEnabled(watcher, e)}
                class="p-1 text-gray-400 hover:text-gray-600 hover:bg-gray-200 rounded transition-colors"
                title={watcher.enabled ? "Pause monitoring" : "Resume monitoring"}
                aria-label={watcher.enabled ? "Pause {watcher.name}" : "Resume {watcher.name}"}
                disabled={togglingId === watcher.id}
              >
                {#if togglingId === watcher.id}
                  <svg class="w-3.5 h-3.5 animate-spin" fill="none" viewBox="0 0 24 24">
                    <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                    <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                  </svg>
                {:else if watcher.enabled}
                  <!-- Pause icon -->
                  <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 9v6m4-6v6m7-3a9 9 0 11-18 0 9 9 0 0118 0z"/>
                  </svg>
                {:else}
                  <!-- Play icon -->
                  <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z"/>
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/>
                  </svg>
                {/if}
              </button>
              <button
                onclick={(e) => { e.stopPropagation(); onedit(watcher); }}
                class="p-1 text-gray-400 hover:text-gray-600 hover:bg-gray-200 rounded transition-colors"
                title="Edit watcher"
                aria-label="Edit {watcher.name}"
              >
                <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z"/>
                </svg>
              </button>
              <button
                onclick={(e) => { e.stopPropagation(); ondelete(watcher); }}
                class="p-1 text-gray-400 hover:text-red-600 hover:bg-red-50 rounded transition-colors"
                title="Delete watcher"
                aria-label="Delete {watcher.name}"
              >
                <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"/>
                </svg>
              </button>
            </div>
          </div>
        </div>
      {/each}
    {/if}
  </div>

  <!-- Footer Version -->
  <div class="p-4 border-t border-gray-200">
    <p class="text-[10px] text-gray-400">v{version}</p>
  </div>
</aside>
