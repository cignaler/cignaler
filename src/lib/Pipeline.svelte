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

    // Dynamic border color based on status
    let borderColorClass = $derived(
        state === "success" ? "border-l-green-500 hover:border-green-400" :
        state === "failed" ? "border-l-red-500 hover:border-red-400" :
        "border-l-yellow-500 hover:border-yellow-400"
    );
</script>

<div class="
    flex w-full justify-between items-center
    my-3 px-6 py-5
    bg-white/90 backdrop-blur-md
    rounded-xl shadow-md hover:shadow-2xl
    border border-l-4 border-gray-100
    {borderColorClass}
    transition-all duration-300 ease-out
    hover:scale-[1.01] hover:-translate-y-0.5
    cursor-pointer group
    relative overflow-hidden
">
    <!-- Subtle gradient overlay on hover -->
    <div class="
        absolute inset-0
        bg-gradient-to-r from-primary-50/0 via-primary-50/30 to-primary-50/0
        opacity-0 group-hover:opacity-100
        transition-opacity duration-300
        pointer-events-none
    "></div>

    <!-- Content -->
    <div class="flex basis-1/6 w-36 relative z-10">
        <Badge color={stateColor} rounded class="px-4 py-2 self-center font-semibold">
            <Indicator color={stateColor} size="sm" class="mr-2"/>{state}
        </Badge>
    </div>
    <p
        onclick={gotoWeb}
        class="
            flex basis-1/3 grow px-6 justify-start items-center self-center
            font-bold text-gray-800 text-base
            hover:text-primary-600 transition-colors
            relative z-10
        "
    >
        {name}
    </p>
    <p class="
        flex basis-1/3 grow px-6 items-center self-center
        text-gray-600 text-sm font-medium font-mono
        relative z-10
    ">
        {formatDate(lastExecuted)}
    </p>
    <Checkbox checked class="flex self-center relative z-10"/>
</div>
