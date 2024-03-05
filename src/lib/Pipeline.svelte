<script lang="ts">
    import {Badge, Card, Checkbox, Indicator} from 'flowbite-svelte';
    import { open } from '@tauri-apps/api/shell';

    export let webUrl: String;
    export let name: String;
    export let state: String;
    export let lastExecuted: String | null = null

    function setColor(state: String) :  "green" | "red" | "yellow" {
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

    function formatDate(date: String) {
        let d = new Date(date)
        return d.toLocaleString()
    }

    let stateColor = setColor(state)
</script>

<div on:click={gotoWeb} class="flex w-full justify-between align-middle my-4 px-6 py-4 bg-white rounded-md">
    <div class="flex basis-1/6 w-36">
        <Badge color={stateColor} rounded class="px-4 py-2 self-center">
            <Indicator color={stateColor} size="sm" class="mr-2"/>{state}
        </Badge>
    </div>
    <p class="flex basis-1/3 grow px-6 justify-start align-middle self-center font-bold">{name}</p>
    <p class="flex basis-1/3 grow px-6 align-middle self-center">{formatDate(lastExecuted)}</p>
    <Checkbox checked class="flex self-center"/>
</div>
