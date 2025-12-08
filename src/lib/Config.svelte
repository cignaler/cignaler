<script lang="ts">
    import { Badge, Button } from 'flowbite-svelte';
    import { EditOutline } from 'flowbite-svelte-icons';
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

<div class="flex w-full justify-between align-middle my-4 px-6 py-4 bg-white rounded-md shadow-sm hover:shadow-md transition-shadow">
    <div class="flex items-center gap-4 flex-1">
        <div class="flex items-center justify-center w-12 h-12 bg-gray-100 rounded-lg text-2xl">
            {getServerEmoji(serverType)}
        </div>
        <div class="flex flex-col">
            <p class="font-bold text-gray-900">{name}</p>
            <p onclick={gotoWeb} class="text-sm text-blue-600 hover:underline cursor-pointer">{urlString}</p>
        </div>
    </div>
    <div class="flex items-center gap-4">
        <Badge color="blue" rounded class="px-3 py-1">
            {serverType}
        </Badge>
        <div class="text-xs text-gray-500 font-mono">
            {maskSecret(secret)}
        </div>
        <Button size="xs" color="light" onclick={handleEdit}>
            <EditOutline class="w-4 h-4" />
        </Button>
    </div>
</div>
