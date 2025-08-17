<script lang="ts">
    import { Badge, Card, Checkbox, Indicator } from 'flowbite-svelte';
    import { open } from '@tauri-apps/plugin-shell';

    export let urlString: string;
    export let name: string;
    export let secret: string;

    function setColor(state: string): "green" | "red" | "yellow" {
        if (state === "success") {
            return "green"
        } else if (state === "failed") {
            return "red"
        } else {
            return "yellow"
        }
    }

    function gotoWeb() {
        open(urlString)
    }

    let stateColor = setColor("green")
</script>

<div class="flex w-full justify-between align-middle my-4 px-6 py-4 bg-white rounded-md">
    <div class="flex basis-1/6 w-36">
        <Badge color={stateColor} rounded class="px-4 py-2 self-center">
            <Indicator color={stateColor} size="sm" class="mr-2"/>{stateColor}
        </Badge>
    </div>
    <p on:click={gotoWeb} class="flex basis-1/3 grow px-6 justify-start align-middle self-center font-bold">{name}</p>
    <Checkbox checked class="flex self-center"/>
</div>
