<script lang="ts">
  import { onDestroy } from "svelte";
  import Label from "./lib/components/ui/Label.svelte";
  import Input from "./lib/components/ui/Input.svelte";
  import Select from "./lib/components/ui/Select.svelte";
  import Button from "./lib/components/ui/Button.svelte";
  import Modal from "./lib/components/ui/Modal.svelte";
  import ConfirmDialog from "./lib/components/ui/ConfirmDialog.svelte";
  import ToastContainer from "./lib/components/ui/ToastContainer.svelte";
  import Pipelines from "./lib/Pipelines.svelte";
  import WatchersSidebar from "./lib/WatchersSidebar.svelte";
  import PreferencesModal from "./lib/PreferencesModal.svelte";
  import { addWatcher, updateWatcher, deleteWatcher, loadWatchers, serversState, watchersState, type ProjectWatcher } from "./lib/stores/watchers.svelte";
  import { toast } from "./lib/stores/toast.svelte";
  import { invoke } from "@tauri-apps/api/core";

  let selectedWatcherId = $state<number | null>(null);
  let pipelineWatcherModalState = $state(false);
  let preferencesModalState = $state(false);
  let editingWatcherId = $state<number | null>(null);

  // Confirm dialog state for watcher deletion
  let confirmDeleteWatcherOpen = $state(false);
  let watcherToDelete = $state<ProjectWatcher | null>(null);
  let deletingWatcherId = $state<number | null>(null);

  // Form data for pipeline watcher modal
  let watcherName = $state("");
  let watcherCiServer = $state("");
  let watcherProjectPath = $state("");
  let watcherTag = $state("");
  let availableTags = $state<string[]>([]);
  let loadingTags = $state(false);
  let tagsError = $state("");
  let loadTagsTimeout = $state<ReturnType<typeof setTimeout> | null>(null);

  // Load watchers on app startup
  $effect(() => {
    loadWatchers();
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

  function showPipelineWatcherModal() {
    clearPipelineWatcherForm();
    const firstServer = serversState.servers[0];
    if (firstServer) {
      watcherCiServer = firstServer.name;
    }
    pipelineWatcherModalState = true;
  }

  function showPreferencesModal() {
    preferencesModalState = true;
  }

  function clearPipelineWatcherForm() {
    watcherName = "";
    watcherCiServer = "";
    watcherProjectPath = "";
    watcherTag = "";
    availableTags = [];
    loadingTags = false;
    tagsError = "";
    editingWatcherId = null;
  }

  function handleEditWatcher(watcher: ProjectWatcher) {
    // Set editing state first to prevent auto-load effect from running
    editingWatcherId = watcher.id;
    // Pre-fill form with watcher data
    watcherName = watcher.name;
    watcherCiServer = watcher.ci_server_name;
    watcherProjectPath = watcher.project_path;
    watcherTag = watcher.default_branch || "";
    availableTags = [];
    loadingTags = false;
    tagsError = "";
    pipelineWatcherModalState = true;
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

  async function loadTags() {
    if (!watcherCiServer || !watcherProjectPath.trim()) {
      availableTags = [];
      return;
    }

    try {
      loadingTags = true;
      tagsError = "";
      const refs = await invoke<string[]>("get_pipeline_references", {
        ciServerName: watcherCiServer,
        projectName: watcherProjectPath.trim(),
      });
      availableTags = refs;
    } catch (error) {
      console.error("Failed to load tags:", error);
      tagsError = String(error);
      availableTags = [];
    } finally {
      loadingTags = false;
    }
  }

  function debouncedLoadTags() {
    if (loadTagsTimeout) {
      clearTimeout(loadTagsTimeout);
    }
    loadTagsTimeout = setTimeout(loadTags, 800);
  }

  // Cleanup timeout on component destroy
  onDestroy(() => {
    if (loadTagsTimeout) {
      clearTimeout(loadTagsTimeout);
    }
  });

  // Auto-load tags when server or project path changes (only when adding new watcher)
  $effect(() => {
    // Track these values
    const server = watcherCiServer;
    const path = watcherProjectPath;
    const isOpen = pipelineWatcherModalState;
    const isEditing = editingWatcherId;

    if (!isOpen || isEditing) return;

    if (server && path.trim().length > 0) {
      debouncedLoadTags();
    } else {
      availableTags = [];
      watcherTag = "";
    }
  });
  async function savePipelineWatcher() {
    const isEditing = editingWatcherId !== null;
    console.log(`${isEditing ? 'Updating' : 'Saving'} pipeline watcher:`, {
      id: editingWatcherId,
      name: watcherName,
      ciServer: watcherCiServer,
      projectPath: watcherProjectPath,
      tag: watcherTag
    });

    if (!watcherName.trim() || !watcherCiServer || !watcherProjectPath.trim() || !watcherTag.trim()) {
      toast.warning("Please fill in all required fields including tag");
      return;
    }

    try {
      if (isEditing && editingWatcherId !== null) {
        await updateWatcher(
          editingWatcherId,
          watcherName.trim(),
          watcherCiServer,
          watcherProjectPath.trim(),
          watcherTag.trim()
        );
        toast.success("Watcher updated successfully");
      } else {
        await addWatcher(
          watcherName.trim(),
          watcherCiServer,
          watcherProjectPath.trim(),
          watcherTag.trim()
        );
        toast.success("Watcher added successfully");
      }

      pipelineWatcherModalState = false;
      clearPipelineWatcherForm();
    } catch (error) {
      console.error(`Failed to ${isEditing ? 'update' : 'add'} pipeline watcher:`, error);
      toast.error(`Failed to ${isEditing ? 'update' : 'add'} watcher: ` + error);
    }
  }
</script>

<main class="flex h-screen bg-gray-50">
  <!-- Sidebar -->
  <WatchersSidebar
    onadd={showPipelineWatcherModal}
    onedit={handleEditWatcher}
    ondelete={handleDeleteWatcher}
    selectedWatcherId={selectedWatcherId}
    onselect={handleSelectWatcher}
  />

  <!-- Main Content -->
  <div class="flex-1 flex flex-col overflow-hidden">
    <Pipelines selectedWatcherId={selectedWatcherId} onpreferences={showPreferencesModal} />
  </div>

  <!-- Add/Edit Pipeline Watcher Modal -->
  <Modal title={editingWatcherId ? "Edit Pipeline Watcher" : "Add Pipeline Watcher"} bind:open={pipelineWatcherModalState} autoclose>
      {#snippet children()}
        <div class="mb-6">
          <Label for="watcher-name" class="block mb-2">Watcher Name *</Label>
          <Input
            type="text"
            id="watcher-name"
            placeholder="e.g., My Project Main Branch"
            bind:value={watcherName}
            required
          />
        </div>

        <div class="mb-6">
          <Label for="watcher-ci-server" class="block mb-2">CI Server *</Label>
          <Select id="watcher-ci-server" bind:value={watcherCiServer}>
            {#if serversState.servers.length === 0}
              <option value="">No CI servers available</option>
            {:else}
              {#each serversState.servers as server (server.name)}
                <option value={server.name}>{server.name} ({server.server_type})</option>
              {/each}
            {/if}
          </Select>
          {#if serversState.servers.length === 0}
            <p class="text-xs text-red-500 mt-1">Please add a CI server first</p>
          {/if}
        </div>

        <div class="mb-6">
          <Label for="watcher-project-path" class="block mb-2">Project Path *</Label>
          <Input
            type="text"
            id="watcher-project-path"
            placeholder="e.g., owner/project-name"
            bind:value={watcherProjectPath}
            required
          />
          <p class="text-xs text-gray-500 mt-1">The project path as it appears in your CI server</p>
        </div>

        <div class="mb-6">
          <Label for="watcher-tag" class="block mb-2">Tag/Branch *</Label>
          <div class="relative">
            <Input
              type="text"
              id="watcher-tag"
              placeholder={loadingTags ? "Loading tags..." : "Type to search tags..."}
              bind:value={watcherTag}
              disabled={loadingTags || !watcherCiServer || !watcherProjectPath.trim()}
              list="tags-list"
              required
            />
            {#if !loadingTags && watcherCiServer && watcherProjectPath.trim()}
              <Button
                size="xs"
                color="light"
                class="absolute right-1 top-1"
                onclick={loadTags}
              >
                Refresh
              </Button>
            {/if}
          </div>
          <datalist id="tags-list">
            {#each availableTags as tag (tag)}
              <option value={tag}>{tag}</option>
            {/each}
          </datalist>
          {#if tagsError}
            <p class="text-xs text-red-500 mt-1">Error loading tags: {tagsError}</p>
          {:else if availableTags.length > 0}
            <p class="text-xs text-gray-500 mt-1">{availableTags.length} tags/branches available</p>
          {:else if !watcherCiServer || !watcherProjectPath.trim()}
            <p class="text-xs text-gray-500 mt-1">Select a server and enter project path first</p>
          {/if}
        </div>
      {/snippet}

      {#snippet footer()}
        <Button color="primary" onclick={savePipelineWatcher} disabled={serversState.servers.length === 0}>
          {editingWatcherId ? "Save Changes" : "Add Watcher"}
        </Button>
        <Button color="alternative" onclick={() => { pipelineWatcherModalState = false; clearPipelineWatcherForm(); }}>Cancel</Button>
      {/snippet}
    </Modal>

  <!-- Preferences Modal -->
  <PreferencesModal bind:open={preferencesModalState} />

  <!-- Confirm Delete Watcher Dialog -->
  <ConfirmDialog
    bind:open={confirmDeleteWatcherOpen}
    title="Delete Watcher"
    message={`Are you sure you want to delete the watcher "${watcherToDelete?.name}"?`}
    confirmText="Delete"
    variant="danger"
    loading={deletingWatcherId !== null}
    onconfirm={confirmDeleteWatcher}
  />

  <!-- Toast Container -->
  <ToastContainer />
</main>
