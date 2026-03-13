<script lang="ts">
    import Label from "./components/ui/Label.svelte";
    import Input from "./components/ui/Input.svelte";
    import Dropdown from "./components/ui/Dropdown.svelte";
    import Combobox from "./components/ui/Combobox.svelte";
    import Button from "./components/ui/Button.svelte";
    import Modal from "./components/ui/Modal.svelte";
    import {
        serversState,
        addWatcher,
        updateWatcher,
        loadRefs,
        getCachedRefs,
        isRefsLoading,
        getRefsError,
        refreshRefsInBackground,
        type ProjectWatcher
    } from "./stores/watchers.svelte";
    import { toast } from "./stores/toast.svelte";

    interface Props {
        open: boolean;
        editingWatcher?: ProjectWatcher | null;
        onsave?: () => void;
        onclose?: () => void;
    }

    let {
        open = $bindable(false),
        editingWatcher = null,
        onsave,
        onclose
    }: Props = $props();

    // Form state (consolidated)
    let form = $state({
        name: '',
        ciServer: '',
        projectPath: '',
        tag: ''
    });

    // Form validation errors
    let errors = $state({
        name: '',
        ciServer: '',
        projectPath: '',
        tag: ''
    });

    // UI state
    let saving = $state(false);
    let refsInitialized = $state(false);

    // Derived state
    let isEditing = $derived(editingWatcher !== null);
    let modalTitle = $derived(isEditing ? 'Edit Pipeline Watcher' : 'Add Pipeline Watcher');

    // Server options for dropdown
    let serverOptions = $derived(
        serversState.servers.map(server => ({
            value: server.name,
            label: server.name,
            description: `${server.server_type} - ${server.url_string}`,
            icon: getServerEmoji(server.server_type)
        }))
    );

    // Refs state from cache
    let availableRefs = $derived(
        form.ciServer && form.projectPath.trim()
            ? getCachedRefs(form.ciServer, form.projectPath.trim())
            : []
    );

    let loadingRefs = $derived(
        form.ciServer && form.projectPath.trim()
            ? isRefsLoading(form.ciServer, form.projectPath.trim())
            : false
    );

    let refsError = $derived(
        form.ciServer && form.projectPath.trim()
            ? getRefsError(form.ciServer, form.projectPath.trim())
            : undefined
    );

    function getServerEmoji(type: string): string {
        switch (type.toLowerCase()) {
            case 'gitlab': return '🦊';
            case 'github': return '🐙';
            case 'jenkins': return '🔧';
            default: return '🖥️';
        }
    }

    function resetForm() {
        form = {
            name: '',
            ciServer: serversState.servers[0]?.name || '',
            projectPath: '',
            tag: ''
        };
        errors = {
            name: '',
            ciServer: '',
            projectPath: '',
            tag: ''
        };
        refsInitialized = false;
    }

    function populateFromWatcher(watcher: ProjectWatcher) {
        form = {
            name: watcher.name,
            ciServer: watcher.ci_server_name,
            projectPath: watcher.project_path,
            tag: watcher.default_branch || ''
        };
        errors = {
            name: '',
            ciServer: '',
            projectPath: '',
            tag: ''
        };
        refsInitialized = true; // Don't auto-load when editing
    }

    function validateForm(): boolean {
        let valid = true;

        // Reset errors
        errors = {
            name: '',
            ciServer: '',
            projectPath: '',
            tag: ''
        };

        if (!form.name.trim()) {
            errors.name = 'Watcher name is required';
            valid = false;
        }

        if (!form.ciServer) {
            errors.ciServer = 'Please select a CI server';
            valid = false;
        }

        if (!form.projectPath.trim()) {
            errors.projectPath = 'Project path is required';
            valid = false;
        }

        if (!form.tag.trim()) {
            errors.tag = 'Please select or enter a branch/tag';
            valid = false;
        }

        return valid;
    }

    async function handleSave() {
        if (!validateForm()) {
            return;
        }

        try {
            saving = true;

            if (isEditing && editingWatcher) {
                await updateWatcher(
                    editingWatcher.id,
                    form.name.trim(),
                    form.ciServer,
                    form.projectPath.trim(),
                    form.tag.trim()
                );
                toast.success('Watcher updated successfully');
            } else {
                await addWatcher(
                    form.name.trim(),
                    form.ciServer,
                    form.projectPath.trim(),
                    form.tag.trim()
                );
                toast.success('Watcher added successfully');
            }

            open = false;
            onsave?.();
        } catch (error) {
            console.error(`Failed to ${isEditing ? 'update' : 'add'} watcher:`, error);
            toast.error(`Failed to ${isEditing ? 'update' : 'add'} watcher: ${error}`);
        } finally {
            saving = false;
        }
    }

    function handleClose() {
        open = false;
        onclose?.();
    }

    async function handleLoadRefs() {
        if (!form.ciServer || !form.projectPath.trim()) return;

        // Force refresh from server
        await loadRefs(form.ciServer, form.projectPath.trim(), true);
    }

    // Debounce timer for project path changes
    let debounceTimer: ReturnType<typeof setTimeout> | null = null;

    function debouncedLoadRefs() {
        if (debounceTimer) {
            clearTimeout(debounceTimer);
        }
        debounceTimer = setTimeout(() => {
            if (form.ciServer && form.projectPath.trim()) {
                loadRefs(form.ciServer, form.projectPath.trim(), false);
            }
        }, 800);
    }

    // Handle modal open/close
    $effect(() => {
        if (open) {
            if (editingWatcher) {
                populateFromWatcher(editingWatcher);
                // Refresh refs in background when editing
                if (editingWatcher.ci_server_name && editingWatcher.project_path) {
                    refreshRefsInBackground(editingWatcher.ci_server_name, editingWatcher.project_path);
                }
            } else {
                resetForm();
            }
        }
    });

    // Auto-load refs when server or project path changes (only for new watchers)
    $effect(() => {
        const server = form.ciServer;
        const path = form.projectPath;

        // Skip if editing or if not yet initialized
        if (isEditing && refsInitialized) return;

        if (server && path.trim()) {
            debouncedLoadRefs();
        }
    });

    // Cleanup on unmount
    $effect(() => {
        return () => {
            if (debounceTimer) {
                clearTimeout(debounceTimer);
            }
        };
    });

    // Generate help text for refs combobox
    let refsHelpText = $derived.by(() => {
        if (!form.ciServer || !form.projectPath.trim()) {
            return 'Select a server and enter project path first';
        }
        if (loadingRefs) {
            return 'Loading branches and tags...';
        }
        if (availableRefs.length > 0) {
            return `${availableRefs.length} branches/tags available`;
        }
        return 'Enter project path to load available branches';
    });
</script>

<Modal title={modalTitle} bind:open={open} autoclose size="md">
    {#snippet children()}
        <div class="space-y-5">
            <!-- Watcher Name -->
            <div>
                <Label for="watcher-name" class="block mb-2">
                    Watcher Name
                    <span class="text-red-500 ml-0.5" aria-hidden="true">*</span>
                </Label>
                <Input
                    type="text"
                    id="watcher-name"
                    placeholder="e.g., My Project Main Branch"
                    bind:value={form.name}
                    error={errors.name}
                    required
                />
            </div>

            <!-- CI Server -->
            <div>
                <Label for="watcher-ci-server" class="block mb-2">
                    CI Server
                    <span class="text-red-500 ml-0.5" aria-hidden="true">*</span>
                </Label>
                {#if serversState.servers.length === 0}
                    <div class="p-4 bg-amber-50 border border-amber-200 rounded-lg">
                        <div class="flex items-start gap-3">
                            <svg class="w-5 h-5 text-amber-500 shrink-0 mt-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"/>
                            </svg>
                            <div>
                                <p class="text-sm font-medium text-amber-800">No CI servers configured</p>
                                <p class="text-xs text-amber-600 mt-1">
                                    Please add a CI server in Preferences before creating a watcher.
                                </p>
                            </div>
                        </div>
                    </div>
                {:else}
                    <Dropdown
                        id="watcher-ci-server"
                        bind:value={form.ciServer}
                        options={serverOptions}
                        placeholder="Select a CI server"
                    />
                    {#if errors.ciServer}
                        <p class="text-xs text-red-500 mt-1" role="alert">{errors.ciServer}</p>
                    {/if}
                {/if}
            </div>

            <!-- Project Path -->
            <div>
                <Label for="watcher-project-path" class="block mb-2">
                    Project Path
                    <span class="text-red-500 ml-0.5" aria-hidden="true">*</span>
                </Label>
                <Input
                    type="text"
                    id="watcher-project-path"
                    placeholder="e.g., owner/project-name"
                    bind:value={form.projectPath}
                    error={errors.projectPath}
                    helpText="The project path as it appears in your CI server"
                    required
                />
            </div>

            <!-- Tag/Branch -->
            <div>
                <Label for="watcher-tag" class="block mb-2">
                    Branch / Tag
                    <span class="text-red-500 ml-0.5" aria-hidden="true">*</span>
                </Label>
                <Combobox
                    id="watcher-tag"
                    bind:value={form.tag}
                    options={availableRefs}
                    placeholder={loadingRefs ? 'Loading...' : 'Type to search or select...'}
                    disabled={!form.ciServer || !form.projectPath.trim()}
                    loading={loadingRefs}
                    error={errors.tag || (refsError ? `Error: ${refsError}` : '')}
                    helpText={refsHelpText}
                    onrefresh={handleLoadRefs}
                />
            </div>
        </div>
    {/snippet}

    {#snippet footer()}
        <Button
            color="primary"
            onclick={handleSave}
            disabled={saving || serversState.servers.length === 0}
        >
            {#if saving}
                <svg class="w-4 h-4 mr-2 animate-spin" fill="none" viewBox="0 0 24 24">
                    <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                    <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                </svg>
                Saving...
            {:else}
                {isEditing ? 'Save Changes' : 'Add Watcher'}
            {/if}
        </Button>
        <Button color="alternative" onclick={handleClose} disabled={saving}>
            Cancel
        </Button>
    {/snippet}
</Modal>
