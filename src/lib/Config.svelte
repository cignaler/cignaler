<script lang="ts">
    import Badge from './components/ui/Badge.svelte';
    import Button from './components/ui/Button.svelte';
    import EditIcon from './components/icons/EditIcon.svelte';
    import { open } from '@tauri-apps/plugin-shell';

    let {
        urlString,
        name,
        secret,
        serverType = 'gitlab',
        onedit
    }: {
        urlString: string;
        name: string;
        secret: string;
        serverType?: string;
        onedit?: (detail: { name: string; urlString: string; secret: string; serverType: string }) => void;
    } = $props();

    function gotoWeb() {
        open(urlString)
    }

    function handleEdit() {
        onedit?.({
            name,
            urlString,
            secret,
            serverType
        });
    }

    function getServerEmoji(type: string): string {
        switch(type.toLowerCase()) {
            case 'gitlab': return '🦊';
            case 'github': return '🐙';
            case 'jenkins': return '⚙️';
            default: return '🖥️';
        }
    }

    function maskSecret(secret: string): string {
        if (!secret || secret.length < 8) return '••••••••';
        return secret.substring(0, 4) + '••••••••' + secret.substring(secret.length - 4);
    }
</script>

<div class="
    flex w-full justify-between items-center
    my-4 px-6 py-5
    bg-white/90 backdrop-blur-md
    rounded-xl shadow-lg hover:shadow-2xl
    border border-l-4 border-gray-100 border-l-secondary-400
    hover:border-l-secondary-500
    transition-all duration-300 ease-out
    hover:scale-[1.01] hover:-translate-y-0.5
    group
    relative overflow-hidden
">
    <!-- Subtle gradient overlay on hover -->
    <div class="
        absolute inset-0
        bg-gradient-to-r from-secondary-50/0 via-secondary-50/30 to-secondary-50/0
        opacity-0 group-hover:opacity-100
        transition-opacity duration-300
        pointer-events-none
    "></div>

    <div class="flex items-center gap-5 flex-1 relative z-10">
        <div class="
            flex items-center justify-center
            w-14 h-14
            bg-gradient-to-br from-gray-50 to-gray-100
            rounded-xl text-3xl
            shadow-sm
            group-hover:shadow-md
            transition-shadow duration-300
        ">
            {getServerEmoji(serverType)}
        </div>
        <div class="flex flex-col gap-1">
            <p class="font-bold text-gray-900 text-lg">{name}</p>
            <button
                type="button"
                onclick={gotoWeb}
                class="
                    text-sm text-secondary-600 hover:text-secondary-700
                    hover:underline cursor-pointer
                    font-medium transition-colors
                    font-mono text-left
                "
            >
                {urlString}
            </button>
        </div>
    </div>
    <div class="flex items-center gap-4 relative z-10">
        <Badge color="blue" rounded class="px-3 py-1.5 font-semibold uppercase text-xs">
            {serverType}
        </Badge>
        <div class="
            text-xs text-gray-500 font-mono
            bg-gray-50 px-3 py-1.5 rounded-lg
            border border-gray-200
        ">
            {maskSecret(secret)}
        </div>
        <Button size="xs" color="light" onclick={handleEdit}>
            <EditIcon class="w-4 h-4" size={4} />
        </Button>
    </div>
</div>
