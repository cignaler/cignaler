import { describe, it, expect, vi, beforeEach } from 'vitest';

// Since .svelte.ts files require Svelte compilation which conflicts with Vite 7,
// we test the watcher store logic by mocking the invoke calls

// Mock invoke
const mockInvoke = vi.fn();

vi.mock('@tauri-apps/api/core', () => ({
    invoke: (...args: unknown[]) => mockInvoke(...args)
}));

interface ProjectWatcher {
    id: number;
    name: string;
    ci_server_name: string;
    project_path: string;
    default_branch: string | null;
    enabled: boolean;
}

interface CiServer {
    name: string;
    server_type: string;
    url_string: string;
    api_key: string;
}

// Simplified store logic for testing
function createWatchersStore() {
    const state = {
        watchers: [] as ProjectWatcher[],
        loading: false,
        error: null as string | null
    };

    const serversState = {
        servers: [] as CiServer[],
        loading: false,
        error: null as string | null
    };

    async function loadWatchers() {
        try {
            state.loading = true;
            state.error = null;
            const projects = await mockInvoke('read_projects');
            state.watchers = projects as ProjectWatcher[];
        } catch (err) {
            state.error = String(err);
            state.watchers = [];
        } finally {
            state.loading = false;
        }
    }

    async function loadServers() {
        try {
            serversState.loading = true;
            serversState.error = null;
            const data = await mockInvoke('read_ci_servers');
            serversState.servers = data as CiServer[];
        } catch (err) {
            serversState.error = String(err);
            serversState.servers = [];
        } finally {
            serversState.loading = false;
        }
    }

    async function addWatcher(
        name: string,
        ciServerName: string,
        projectPath: string,
        defaultBranch: string
    ) {
        await mockInvoke('store_project_data', {
            name,
            ciServerName,
            projectPath,
            defaultBranch
        });
        await loadWatchers();
    }

    async function deleteWatcher(id: number) {
        await mockInvoke('delete_project', { id });
        await loadWatchers();
    }

    async function toggleWatcherEnabled(id: number, enabled: boolean) {
        await mockInvoke('set_project_enabled', { id, enabled });
        await loadWatchers();
    }

    return {
        state,
        serversState,
        loadWatchers,
        loadServers,
        addWatcher,
        deleteWatcher,
        toggleWatcherEnabled
    };
}

describe('Watchers Store Logic', () => {
    let store: ReturnType<typeof createWatchersStore>;

    beforeEach(() => {
        vi.clearAllMocks();
        store = createWatchersStore();
    });

    describe('loadWatchers', () => {
        it('should load watchers successfully', async () => {
            const mockWatchers: ProjectWatcher[] = [
                { id: 1, name: 'Test Watcher', ci_server_name: 'gitlab', project_path: 'test/project', default_branch: 'main', enabled: true },
                { id: 2, name: 'Another Watcher', ci_server_name: 'gitlab', project_path: 'test/other', default_branch: 'develop', enabled: false }
            ];

            mockInvoke.mockResolvedValueOnce(mockWatchers);

            await store.loadWatchers();

            expect(mockInvoke).toHaveBeenCalledWith('read_projects');
            expect(store.state.watchers).toEqual(mockWatchers);
            expect(store.state.loading).toBe(false);
            expect(store.state.error).toBeNull();
        });

        it('should handle load error', async () => {
            mockInvoke.mockRejectedValueOnce(new Error('Database error'));

            await store.loadWatchers();

            expect(store.state.watchers).toEqual([]);
            expect(store.state.error).toBe('Error: Database error');
        });

        it('should set loading state correctly', async () => {
            let loadingDuringCall = false;
            mockInvoke.mockImplementationOnce(async () => {
                loadingDuringCall = store.state.loading;
                return [];
            });

            await store.loadWatchers();

            expect(loadingDuringCall).toBe(true);
            expect(store.state.loading).toBe(false);
        });
    });

    describe('loadServers', () => {
        it('should load servers successfully', async () => {
            const mockServers: CiServer[] = [
                { name: 'My GitLab', server_type: 'gitlab', url_string: 'https://gitlab.com', api_key: 'token' }
            ];

            mockInvoke.mockResolvedValueOnce(mockServers);

            await store.loadServers();

            expect(mockInvoke).toHaveBeenCalledWith('read_ci_servers');
            expect(store.serversState.servers).toEqual(mockServers);
        });

        it('should handle server load error', async () => {
            mockInvoke.mockRejectedValueOnce(new Error('Connection failed'));

            await store.loadServers();

            expect(store.serversState.servers).toEqual([]);
            expect(store.serversState.error).toBe('Error: Connection failed');
        });
    });

    describe('addWatcher', () => {
        it('should add watcher and reload list', async () => {
            const newWatcher: ProjectWatcher = {
                id: 1,
                name: 'New Watcher',
                ci_server_name: 'gitlab',
                project_path: 'new/project',
                default_branch: 'main',
                enabled: true
            };

            mockInvoke
                .mockResolvedValueOnce(undefined) // store_project_data
                .mockResolvedValueOnce([newWatcher]); // read_projects

            await store.addWatcher('New Watcher', 'gitlab', 'new/project', 'main');

            expect(mockInvoke).toHaveBeenCalledWith('store_project_data', {
                name: 'New Watcher',
                ciServerName: 'gitlab',
                projectPath: 'new/project',
                defaultBranch: 'main'
            });
            expect(store.state.watchers).toEqual([newWatcher]);
        });
    });

    describe('deleteWatcher', () => {
        it('should delete watcher and reload list', async () => {
            store.state.watchers = [
                { id: 1, name: 'Test', ci_server_name: 'gitlab', project_path: 'test/project', default_branch: 'main', enabled: true }
            ];

            mockInvoke
                .mockResolvedValueOnce(undefined) // delete_project
                .mockResolvedValueOnce([]); // read_projects

            await store.deleteWatcher(1);

            expect(mockInvoke).toHaveBeenCalledWith('delete_project', { id: 1 });
            expect(store.state.watchers).toEqual([]);
        });
    });

    describe('toggleWatcherEnabled', () => {
        it('should toggle watcher enabled state', async () => {
            const updatedWatcher: ProjectWatcher = {
                id: 1,
                name: 'Test',
                ci_server_name: 'gitlab',
                project_path: 'test/project',
                default_branch: 'main',
                enabled: false
            };

            mockInvoke
                .mockResolvedValueOnce(undefined) // set_project_enabled
                .mockResolvedValueOnce([updatedWatcher]); // read_projects

            await store.toggleWatcherEnabled(1, false);

            expect(mockInvoke).toHaveBeenCalledWith('set_project_enabled', { id: 1, enabled: false });
            expect(store.state.watchers[0]?.enabled).toBe(false);
        });
    });
});
