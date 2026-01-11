<script lang="ts">
  import Label from "./components/ui/Label.svelte";
  import Input from "./components/ui/Input.svelte";
  import Select from "./components/ui/Select.svelte";
  import Button from "./components/ui/Button.svelte";
  import Modal from "./components/ui/Modal.svelte";
  import ConfirmDialog from "./components/ui/ConfirmDialog.svelte";
  import { serversState, addServer, updateServer, deleteServer, loadServers } from "./stores/watchers.svelte";
  import { toast } from "./stores/toast.svelte";

  interface Props {
    open: boolean;
  }

  let { open = $bindable(false) }: Props = $props();

  // View state: 'list' | 'add' | 'edit'
  let viewState = $state<'list' | 'add' | 'edit'>('list');

  // Form data for adding/editing server
  let serverName = $state("");
  let serverType = $state("gitlab");
  let serverUrl = $state("");
  let serverApiKey = $state("");
  let editingServerName = $state<string | null>(null);

  // Error/loading states
  let saving = $state(false);
  let error = $state("");
  let deletingServerName = $state<string | null>(null);

  // Confirm dialog state
  let confirmDeleteOpen = $state(false);
  let serverToDelete = $state<string | null>(null);

  const serverTypes = [
    { value: "gitlab", label: "GitLab", available: true },
    { value: "github", label: "GitHub Actions", available: false },
    { value: "jenkins", label: "Jenkins", available: false },
  ];

  function resetForm() {
    serverName = "";
    serverType = "gitlab";
    serverUrl = "";
    serverApiKey = "";
    editingServerName = null;
    error = "";
  }

  function showAddForm() {
    resetForm();
    viewState = 'add';
  }

  function showEditForm(server: { name: string; server_type: string; url_string: string; api_key: string }) {
    serverName = server.name;
    serverType = server.server_type;
    serverUrl = server.url_string;
    serverApiKey = server.api_key;
    editingServerName = server.name;
    error = "";
    viewState = 'edit';
  }

  function backToList() {
    resetForm();
    viewState = 'list';
  }

  async function handleSave() {
    if (!serverName.trim() || !serverUrl.trim() || !serverApiKey.trim()) {
      error = "Please fill in all required fields";
      return;
    }

    try {
      saving = true;
      error = "";

      if (viewState === 'add') {
        await addServer(serverName.trim(), serverType, serverUrl.trim(), serverApiKey.trim());
      } else if (viewState === 'edit' && editingServerName) {
        await updateServer(editingServerName, serverType, serverUrl.trim(), serverApiKey.trim());
      }

      backToList();
    } catch (err) {
      error = String(err);
    } finally {
      saving = false;
    }
  }

  function handleDelete(name: string) {
    serverToDelete = name;
    confirmDeleteOpen = true;
  }

  async function confirmDelete() {
    if (!serverToDelete) return;

    const name = serverToDelete;
    deletingServerName = name;

    try {
      await deleteServer(name);
      toast.success(`Server "${name}" deleted successfully`);
    } catch (err) {
      toast.error(`Failed to delete server: ${String(err)}`);
    } finally {
      deletingServerName = null;
      serverToDelete = null;
    }
  }

  function handleClose() {
    resetForm();
    viewState = 'list';
    open = false;
  }

  // Reset state when modal opens
  $effect(() => {
    if (open) {
      resetForm();
      viewState = 'list';
      loadServers();
    }
  });
</script>

<Modal title="Preferences" bind:open={open} autoclose size="lg">
  {#snippet children()}
    {#if viewState === 'list'}
      <!-- Server List View -->
      <div class="space-y-4">
        <div class="flex items-center justify-between">
          <h4 class="text-lg font-semibold text-gray-900">CI Servers</h4>
          <Button size="sm" color="primary" onclick={showAddForm}>
            <svg class="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4"/>
            </svg>
            Add Server
          </Button>
        </div>

        {#if serversState.loading}
          <div class="text-center py-8">
            <p class="text-sm text-gray-500">Loading servers...</p>
          </div>
        {:else if serversState.servers.length === 0}
          <div class="text-center py-8 bg-gray-50 rounded-lg border border-dashed border-gray-300">
            <svg class="w-12 h-12 mx-auto text-gray-400 mb-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M5 12h14M5 12a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v4a2 2 0 01-2 2M5 12a2 2 0 00-2 2v4a2 2 0 002 2h14a2 2 0 002-2v-4a2 2 0 00-2-2m-2-4h.01M17 16h.01"/>
            </svg>
            <p class="text-sm text-gray-500 mb-1">No CI servers configured</p>
            <p class="text-xs text-gray-400">Add a server to start monitoring pipelines</p>
          </div>
        {:else}
          <div class="space-y-2">
            {#each serversState.servers as server (server.name)}
              <div class="flex items-center justify-between p-4 bg-gray-50 rounded-lg border border-gray-200 hover:border-gray-300 transition-colors">
                <div class="flex items-center gap-3">
                  <div class="w-10 h-10 bg-orange-100 rounded-lg flex items-center justify-center">
                    <svg class="w-5 h-5 text-orange-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 12h14M5 12a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v4a2 2 0 01-2 2M5 12a2 2 0 00-2 2v4a2 2 0 002 2h14a2 2 0 002-2v-4a2 2 0 00-2-2"/>
                    </svg>
                  </div>
                  <div>
                    <h5 class="font-medium text-gray-900">{server.name}</h5>
                    <p class="text-xs text-gray-500">
                      {serverTypes.find(t => t.value === server.server_type)?.label || server.server_type} &bull; {server.url_string}
                    </p>
                  </div>
                </div>
                <div class="flex items-center gap-2">
                  <button
                    onclick={() => showEditForm(server)}
                    class="p-2 text-gray-400 hover:text-gray-600 hover:bg-gray-200 rounded-lg transition-colors"
                    title="Edit server"
                    aria-label="Edit {server.name}"
                  >
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z"/>
                    </svg>
                  </button>
                  <button
                    onclick={() => handleDelete(server.name)}
                    class="p-2 text-gray-400 hover:text-red-600 hover:bg-red-50 rounded-lg transition-colors disabled:opacity-50"
                    title="Delete server"
                    aria-label="Delete {server.name}"
                    disabled={deletingServerName === server.name}
                  >
                    {#if deletingServerName === server.name}
                      <svg class="w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24" aria-hidden="true">
                        <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                        <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                      </svg>
                    {:else}
                      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"/>
                      </svg>
                    {/if}
                  </button>
                </div>
              </div>
            {/each}
          </div>
        {/if}

        {#if error}
          <p class="text-sm text-red-500 mt-2">{error}</p>
        {/if}
      </div>
    {:else}
      <!-- Add/Edit Form View -->
      <div class="space-y-4">
        <button
          onclick={backToList}
          class="flex items-center gap-1 text-sm text-gray-500 hover:text-gray-700 transition-colors"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7"/>
          </svg>
          Back to list
        </button>

        <h4 class="text-lg font-semibold text-gray-900">
          {viewState === 'add' ? 'Add CI Server' : 'Edit CI Server'}
        </h4>

        <div class="space-y-4">
          <div>
            <Label for="server-name" class="block mb-2">Server Name *</Label>
            <Input
              type="text"
              id="server-name"
              placeholder="e.g., My GitLab"
              bind:value={serverName}
              disabled={viewState === 'edit'}
              required
            />
            {#if viewState === 'edit'}
              <p class="text-xs text-gray-500 mt-1">Server name cannot be changed</p>
            {/if}
          </div>

          <div>
            <Label for="server-type" class="block mb-2">Server Type *</Label>
            <Select id="server-type" bind:value={serverType}>
              {#each serverTypes as type (type.value)}
                <option value={type.value} disabled={!type.available}>
                  {type.label}{#if !type.available} (Coming Soon){/if}
                </option>
              {/each}
            </Select>
            {#if serverType !== "gitlab"}
              <p class="text-xs text-amber-600 mt-1">This server type is not yet supported</p>
            {/if}
          </div>

          <div>
            <Label for="server-url" class="block mb-2">Server URL *</Label>
            <Input
              type="url"
              id="server-url"
              placeholder="e.g., https://gitlab.com"
              bind:value={serverUrl}
              required
            />
            <p class="text-xs text-gray-500 mt-1">The base URL of your CI server</p>
          </div>

          <div>
            <Label for="server-api-key" class="block mb-2">API Key / Token *</Label>
            <Input
              type="password"
              id="server-api-key"
              placeholder="Enter your API key or personal access token"
              bind:value={serverApiKey}
              required
            />
            <p class="text-xs text-gray-500 mt-1">
              {#if serverType === 'gitlab'}
                Create a personal access token with <code class="bg-gray-100 px-1 rounded">read_api</code> scope
              {:else if serverType === 'github'}
                Create a personal access token with <code class="bg-gray-100 px-1 rounded">repo</code> and <code class="bg-gray-100 px-1 rounded">workflow</code> scopes
              {:else}
                Enter your API token or credentials
              {/if}
            </p>
          </div>

          {#if error}
            <p class="text-sm text-red-500">{error}</p>
          {/if}
        </div>
      </div>
    {/if}
  {/snippet}

  {#snippet footer()}
    {#if viewState === 'list'}
      <Button color="alternative" onclick={handleClose}>Close</Button>
    {:else}
      <Button color="primary" onclick={handleSave} disabled={saving}>
        {saving ? 'Saving...' : (viewState === 'add' ? 'Add Server' : 'Save Changes')}
      </Button>
      <Button color="alternative" onclick={backToList} disabled={saving}>Cancel</Button>
    {/if}
  {/snippet}
</Modal>

<ConfirmDialog
  bind:open={confirmDeleteOpen}
  title="Delete Server"
  message={`Are you sure you want to delete the server "${serverToDelete}"? All watchers using this server will also be deleted.`}
  confirmText="Delete"
  variant="danger"
  loading={deletingServerName !== null}
  onconfirm={confirmDelete}
/>
