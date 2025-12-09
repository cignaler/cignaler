<script lang="ts">
    import Badge from './components/ui/Badge.svelte';
    import Checkbox from './components/ui/Checkbox.svelte';
    import Indicator from './components/ui/Indicator.svelte';
    import { open } from '@tauri-apps/plugin-shell';

    let {
        webUrl,
        name,
        state,
        lastExecuted = null
    }: {
        webUrl: string;
        name: string;
        state: string;
        lastExecuted?: string | null;
    } = $props();

    function setColor(state: string) :  "green" | "red" | "yellow" {
        if (state === "success") {
            return "green"
        } else if (state === "failed") {
            return "red"
        } else {
            return "yellow"
        }
    }

    function gotoWeb() {
        open(webUrl)
    }

    function formatDate(date: string | null) {
        if (!date) return "N/A";
        let d = new Date(date)
        return d.toLocaleString()
    }

    let stateColor = $derived(setColor(state))
</script>

<div class="flex w-full justify-between align-middle my-4 px-6 py-4 bg-white rounded-md">
    <div class="flex basis-1/6 w-36">
        <Badge color={stateColor} rounded class="px-4 py-2 self-center">
            <Indicator color={stateColor} size="sm" class="mr-2"/>{state}
        </Badge>
    </div>
    <p onclick={gotoWeb} class="flex basis-1/3 grow px-6 justify-start align-middle self-center font-bold">{name}</p>
    <p class="flex basis-1/3 grow px-6 align-middle self-center">{formatDate(lastExecuted)}</p>
    <Checkbox checked class="flex self-center"/>
</div>
