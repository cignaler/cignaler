<script lang="ts">
    import { Badge, Button } from 'flowbite-svelte';
    import { Icon } from 'flowbite-svelte-icons';
    import { open } from '@tauri-apps/plugin-shell';
    import { createEventDispatcher } from 'svelte';

    export let urlString: string;
    export let name: string;
    export let secret: string;
    export let serverType: string = 'gitlab';

    const dispatch = createEventDispatcher();

    function gotoWeb() {
        open(urlString)
    }

    function handleEdit() {
        dispatch('edit', {
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
            <p on:click={gotoWeb} class="text-sm text-blue-600 hover:underline cursor-pointer">{urlString}</p>
        </div>
    </div>
    <div class="flex items-center gap-4">
        <Badge color="blue" rounded class="px-3 py-1">
            {serverType}
        </Badge>
        <div class="text-xs text-gray-500 font-mono">
            {maskSecret(secret)}
        </div>
        <Button size="xs" color="light" on:click={handleEdit}>
            <Icon name="edit-outline" class="w-4 h-4" />
        </Button>
    </div>
</div>
