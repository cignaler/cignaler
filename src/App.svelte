<script lang="ts">
  import Label from "./lib/components/ui/Label.svelte";
  import Input from "./lib/components/ui/Input.svelte";
  import Select from "./lib/components/ui/Select.svelte";
  import Button from "./lib/components/ui/Button.svelte";
  import Modal from "./lib/components/ui/Modal.svelte";
  import PullRequestIcon from "./lib/components/icons/PullRequestIcon.svelte";
  import ServerIcon from "./lib/components/icons/ServerIcon.svelte";
  import Pipelines from "./lib/Pipelines.svelte";
  import Configs from "./lib/Configs.svelte";
  import { addWatcher, addServer, updateServer, serversState } from "./lib/stores/watchers.svelte";

  let tabValue = $state("pipelines");
  let activeTab = $derived(tabValue);

  let modalState = $state(false);

  let pipelineWatcherModalState = $state(false);

  // Form data for CI server modal
  let serverName = $state("");
  let serverUrl = $state("");
  let apiToken = $state("");
  let serverType = $state("gitlab");

  // Edit mode tracking
  let isEditMode = $state(false);
  let originalServerName = $state("");

  // Form data for pipeline watcher modal
  let watcherName = $state("");
  let watcherCiServer = $state("");
  let watcherProjectPath = $state("");
  let watcherTag = $state("");
  let availableTags = $state<string[]>([]);
  let loadingTags = $state(false);
  let tagsError = $state("");
  let loadTagsTimeout = $state<ReturnType<typeof setTimeout> | null>(null);

  function setPipelinesActive() {
    tabValue = "pipelines";
  }

  function setConfigActive() {
    tabValue = "ci_servers";
  }

  function showModal() {
    isEditMode = false;
    clearForm();
    modalState = true;
  }

  function showPipelineWatcherModal() {
    clearPipelineWatcherForm();
    if (serversState.servers.length > 0) {
      watcherCiServer = serversState.servers[0].name;
    }
    pipelineWatcherModalState = true;
  }

  function clearPipelineWatcherForm() {
    watcherName = "";
    watcherCiServer = "";
    watcherProjectPath = "";
    watcherTag = "";
    availableTags = [];
    loadingTags = false;
    tagsError = "";
  }

  async function loadTags() {
    if (!watcherCiServer || !watcherProjectPath.trim()) {
      availableTags = [];
      return;
    }

    try {
      loadingTags = true;
      tagsError = "";
      const { invoke } = await import("@tauri-apps/api/core");
      const refs = await invoke<string[]>("get_pipeline_references", {
        ciServerName: watcherCiServer,
        projectName: watcherProjectPath.trim(),
      });
      availableTags = refs;
      console.log(`Loaded ${refs.length} tags/branches`);
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

  // Auto-load tags when server or project path changes
  $effect(() => {
    if (watcherCiServer && watcherProjectPath.trim().length > 0) {
      debouncedLoadTags();
    } else {
      availableTags = [];
      watcherTag = "";
    }
  });

  function handleEdit(detail: { name: string; urlString: string; secret: string; serverType: string }) {
    serverName = detail.name;
    serverUrl = detail.urlString;
    apiToken = detail.secret;
    serverType = detail.serverType;
    originalServerName = detail.name;
    isEditMode = true;
    modalState = true;
  }

  function clearForm() {
    serverName = "";
    serverUrl = "";
    apiToken = "";
    serverType = "gitlab";
    isEditMode = false;
    originalServerName = "";
  }

  async function saveConfig() {
    if (!serverName.trim() || !serverUrl.trim() || !apiToken.trim()) {
      alert("Please fill in all required fields");
      return;
    }

    try {
      if (isEditMode) {
        await updateServer(
          originalServerName,
          serverType,
          serverUrl.trim(),
          apiToken.trim()
        );
        alert("CI server updated successfully!");
      } else {
        await addServer(
          serverName.trim(),
          serverType,
          serverUrl.trim(),
          apiToken.trim()
        );
        alert("CI server added successfully!");
      }

      modalState = false;
      clearForm();
    } catch (error) {
      console.error(`Failed to ${isEditMode ? 'update' : 'save'} CI server:`, error);
      alert(`Failed to ${isEditMode ? 'update' : 'save'} CI server: ` + error);
    }
  }

  async function savePipelineWatcher() {
    console.log("Saving pipeline watcher:", {
      name: watcherName,
      ciServer: watcherCiServer,
      projectPath: watcherProjectPath,
      tag: watcherTag
    });

    if (!watcherName.trim() || !watcherCiServer || !watcherProjectPath.trim() || !watcherTag.trim()) {
      alert("Please fill in all required fields including tag");
      return;
    }

    try {
      await addWatcher(
        watcherName.trim(),
        watcherCiServer,
        watcherProjectPath.trim(),
        watcherTag.trim()
      );

      console.log("Pipeline watcher saved successfully");
      alert("Pipeline watcher added successfully!");
      pipelineWatcherModalState = false;
      clearPipelineWatcherForm();
    } catch (error) {
      console.error("Failed to add pipeline watcher:", error);
      alert("Failed to add pipeline watcher: " + error);
    }
  }
</script>

<main class="min-h-screen">
  <!-- Enhanced Header -->
  <div class="
    sticky top-0 z-40
    bg-white/80 backdrop-blur-lg
    border-b border-gray-200/60
    shadow-sm
  ">
    <div class="flex justify-between items-center py-5 px-8">
      <div class="flex items-center gap-6">
        <h1 class="text-2xl font-bold text-transparent bg-clip-text bg-gradient-to-r from-primary-600 to-secondary-600">
          Cignaler
        </h1>
        <div class="flex gap-3">
          <Button
            color="primary"
            outline={activeTab !== "pipelines"}
            size="md"
            onclick={setPipelinesActive}
          >
            Pipelines
          </Button>
          <Button
            color="primary"
            outline={activeTab !== "ci_servers"}
            size="md"
            onclick={setConfigActive}
          >
            CI Servers
          </Button>
        </div>
      </div>
      <div class="flex gap-3">
        <Button color="primary" onclick={showPipelineWatcherModal} class="gap-2">
          <PullRequestIcon size={5} />
          <span>Add Watcher</span>
        </Button>
        <Button color="primary" onclick={showModal} class="gap-2">
          <ServerIcon size={5} />
          <span>Add Server</span>
        </Button>
      </div>
    </div>
  </div>

  <div class="main-content px-4 py-6">
    {#if activeTab === "pipelines"}
      <Pipelines />
    {:else if activeTab === "ci_servers"}
      <Configs onedit={handleEdit} />
    {/if}
    <Modal title={isEditMode ? "Edit CI server" : "Add new CI server"} bind:open={modalState} autoclose>
      {#snippet children()}
        <div class="mb-6">
          <Label for="server-name" class="block mb-2">Server Name *</Label>
          <Input
            type="text"
            id="server-name"
            placeholder="e.g., My GitLab Server"
            bind:value={serverName}
            disabled={isEditMode}
            required
          />
          {#if isEditMode}
            <p class="text-xs text-gray-500 mt-1">Server name cannot be changed</p>
          {/if}
        </div>
        <div class="mb-6">
          <Label for="server-url" class="block mb-2">Server URL *</Label>
          <Input
            type="url"
            id="server-url"
            placeholder="https://gitlab.example.com"
            bind:value={serverUrl}
            required
          />
        </div>
        <div class="mb-6">
          <Label for="api-token" class="block mb-2">API Token *</Label>
          <Input
            type="password"
            id="api-token"
            placeholder="glpat-xxxxxxxxxxxxxxxxxxxx"
            bind:value={apiToken}
            required
          />
        </div>
        <div class="mb-6">
          <Label for="server-type" class="block mb-2">Server Type</Label>
          <Select id="server-type" bind:value={serverType}>
            <option value="gitlab">GitLab</option>
            <option value="github">GitHub</option>
            <option value="jenkins">Jenkins</option>
          </Select>
        </div>
      {/snippet}

      {#snippet footer()}
        <Button color="primary" onclick={saveConfig}>{isEditMode ? 'Update Server' : 'Add Server'}</Button>
        <Button color="alternative" onclick={() => { modalState = false; clearForm(); }}>Cancel</Button>
      {/snippet}
    </Modal>

    <Modal title="Add Pipeline Watcher" bind:open={pipelineWatcherModalState} autoclose>
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
        <Button color="primary" onclick={savePipelineWatcher} disabled={serversState.servers.length === 0}>Add Watcher</Button>
        <Button color="alternative" onclick={() => { pipelineWatcherModalState = false; clearPipelineWatcherForm(); }}>Cancel</Button>
      {/snippet}
    </Modal>
  </div>
</main>
