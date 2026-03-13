<script lang="ts">
  import ToastContainer from "./lib/components/ui/ToastContainer.svelte";
  import Pipelines from "./lib/Pipelines.svelte";
  import WatchersSidebar from "./lib/WatchersSidebar.svelte";
  import { deleteWatcher, loadWatchers, serversState, watchersState, loadServers, initWatcherAddedListener, type ProjectWatcher } from "./lib/stores/watchers.svelte";
  import { toast } from "./lib/stores/toast.svelte";
  import { initPipelineListener } from "./lib/stores/pipelines.svelte";

  let selectedWatcherId = $state<number | null>(null);
  let watcherModalOpen = $state(false);
  let editingWatcher = $state<ProjectWatcher | null>(null);
  let preferencesModalState = $state(false);

  // Confirm dialog state for watcher deletion
  let confirmDeleteWatcherOpen = $state(false);
  let watcherToDelete = $state<ProjectWatcher | null>(null);
  let deletingWatcherId = $state<number | null>(null);

  // Load watchers and servers on app startup, and start event listeners
  $effect(() => {
    Promise.all([loadWatchers(), loadServers()]);
    initPipelineListener();
    initWatcherAddedListener();
  });

  function handleSelectWatcher(id: number) {
    selectedWatcherId = id;
  }

  // Auto-select first watcher when available
  $effect(() => {
    const watchers = watchersState.watchers;
    if (watchers.length > 0 && selectedWatcherId === null) {
      const firstWatcher = watchers[0];
      if (firstWatcher) {
        selectedWatcherId = firstWatcher.id;
      }
    }
  });

  function showAddWatcherModal() {
    // Prevent opening if no servers configured
    if (serversState.servers.length === 0) {
      toast.warning("Please add a CI server in Preferences first");
      preferencesModalState = true;
      return;
    }

    editingWatcher = null;
    watcherModalOpen = true;
  }

  function showEditWatcherModal(watcher: ProjectWatcher) {
    editingWatcher = watcher;
    watcherModalOpen = true;
  }

  function showPreferencesModal() {
    preferencesModalState = true;
  }

  function handleDeleteWatcher(watcher: ProjectWatcher) {
    watcherToDelete = watcher;
    confirmDeleteWatcherOpen = true;
  }

  async function confirmDeleteWatcher() {
    if (!watcherToDelete) return;

    const watcher = watcherToDelete;
    deletingWatcherId = watcher.id;

    try {
      await deleteWatcher(watcher.id);
      // If we deleted the selected watcher, clear selection
      if (selectedWatcherId === watcher.id) {
        selectedWatcherId = null;
      }
      toast.success(`Watcher "${watcher.name}" deleted successfully`);
    } catch (error) {
      console.error("Failed to delete watcher:", error);
      toast.error("Failed to delete watcher: " + error);
    } finally {
      deletingWatcherId = null;
      watcherToDelete = null;
    }
  }
</script>

<main class="flex h-screen bg-gray-50">
  <!-- Sidebar -->
  <WatchersSidebar
    onadd={showAddWatcherModal}
    onedit={showEditWatcherModal}
    ondelete={handleDeleteWatcher}
    selectedWatcherId={selectedWatcherId}
    onselect={handleSelectWatcher}
  />

  <!-- Main Content -->
  <div class="flex-1 flex flex-col overflow-hidden">
    <Pipelines selectedWatcherId={selectedWatcherId} onpreferences={showPreferencesModal} />
  </div>

  <!-- Add/Edit Pipeline Watcher Modal (lazy) -->
  {#if watcherModalOpen}
    {#await import("./lib/WatcherModal.svelte") then { default: WatcherModal }}
      <WatcherModal bind:open={watcherModalOpen} editingWatcher={editingWatcher} />
    {/await}
  {/if}

  <!-- Preferences Modal (lazy) -->
  {#if preferencesModalState}
    {#await import("./lib/PreferencesModal.svelte") then { default: PreferencesModal }}
      <PreferencesModal bind:open={preferencesModalState} />
    {/await}
  {/if}

  <!-- Confirm Delete Watcher Dialog (lazy) -->
  {#if confirmDeleteWatcherOpen}
    {#await import("./lib/components/ui/ConfirmDialog.svelte") then { default: ConfirmDialog }}
      <ConfirmDialog
        bind:open={confirmDeleteWatcherOpen}
        title="Delete Watcher"
        message={`Are you sure you want to delete the watcher "${watcherToDelete?.name}"?`}
        confirmText="Delete"
        variant="danger"
        loading={deletingWatcherId !== null}
        onconfirm={confirmDeleteWatcher}
      />
    {/await}
  {/if}

  <!-- Toast Container -->
  <ToastContainer />
</main>
