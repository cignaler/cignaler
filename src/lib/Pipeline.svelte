<script lang="ts">
    import { open } from '@tauri-apps/plugin-shell';

    let {
        webUrl,
        name,
        state,
        lastExecuted = null,
        finishedAt = null,
        sha = null,
        source = null
    }: {
        webUrl: string;
        name: string;
        state: string;
        lastExecuted?: string | null;
        finishedAt?: string | null;
        sha?: string | null;
        source?: string | null;
    } = $props();

    function gotoWeb() {
        open(webUrl)
    }

    function formatDate(date: string | null) {
        if (!date) return "N/A";
        let d = new Date(date);
        const now = new Date();
        const diff = now.getTime() - d.getTime();
        const hours = Math.floor(diff / (1000 * 60 * 60));

        if (hours < 24) {
            return d.toLocaleTimeString('en-US', { hour: '2-digit', minute: '2-digit', second: '2-digit', hour12: false });
        } else if (hours < 48) {
            return 'Yesterday, ' + d.toLocaleTimeString('en-US', { hour: '2-digit', minute: '2-digit', hour12: false });
        } else {
            return d.toLocaleDateString('en-US', { month: 'short', day: 'numeric' });
        }
    }

    function formatDuration(startDate: string | null, endDate: string | null): string {
        if (!startDate || !endDate) return "N/A";
        const start = new Date(startDate).getTime();
        const end = new Date(endDate).getTime();
        const diffMs = end - start;

        if (diffMs < 0) return "N/A";

        const totalSeconds = Math.floor(diffMs / 1000);
        const hours = Math.floor(totalSeconds / 3600);
        const minutes = Math.floor((totalSeconds % 3600) / 60);
        const seconds = totalSeconds % 60;

        if (hours > 0) {
            return `${hours}h ${minutes}m ${seconds}s`;
        } else if (minutes > 0) {
            return `${minutes}m ${seconds}s`;
        } else {
            return `${seconds}s`;
        }
    }

    // Commit hash from SHA or extract from ref name
    let commitHash = $derived(() => {
        if (sha) return sha.substring(0, 7);
        // Fallback: check if ref looks like a SHA
        if (name && name.match(/^[a-f0-9]{40}$/i)) return name.substring(0, 7);
        return null;
    });

    // Duration calculated from timestamps
    let duration = $derived(formatDuration(lastExecuted, finishedAt));

    // Dynamic border color based on status
    let borderColorClass = $derived(
        state === "success" ? "border-l-green-500" :
        state === "failed" ? "border-l-red-500" :
        "border-l-yellow-500"
    );
</script>

<div class="
    flex w-full items-center gap-6
    py-5 px-6
    bg-white
    border-b border-l-[5px] border-gray-100
    {borderColorClass}
    hover:bg-gray-50
    transition-colors
">
    <!-- Status Badge -->
    <div class="flex-shrink-0">
        {#if state === "success"}
            <div class="flex items-center gap-1.5 bg-green-50 text-green-600 px-2.5 py-1 rounded text-xs font-medium uppercase">
                <svg class="w-3 h-3" fill="currentColor" viewBox="0 0 20 20">
                    <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd"/>
                </svg>
                Success
            </div>
        {:else if state === "failed"}
            <div class="flex items-center gap-1.5 bg-red-50 text-red-600 px-2.5 py-1 rounded text-xs font-medium uppercase">
                <svg class="w-3 h-3" fill="currentColor" viewBox="0 0 20 20">
                    <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd"/>
                </svg>
                Failed
            </div>
        {:else}
            <div class="flex items-center gap-1.5 bg-yellow-50 text-yellow-600 px-2.5 py-1 rounded text-xs font-medium uppercase">
                <svg class="w-3 h-3 animate-spin" fill="none" viewBox="0 0 24 24">
                    <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                    <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                </svg>
                Running
            </div>
        {/if}
    </div>

    <!-- Pipeline Info -->
    <div class="flex-1 min-w-0">
        <h3 class="text-sm font-normal text-gray-900 mb-1.5">{name}</h3>
        <div class="flex items-center gap-3 text-xs text-gray-500">
            <!-- Source/Trigger -->
            {#if source}
            <div class="flex items-center gap-1.5">
                <svg class="w-3 h-3" fill="currentColor" viewBox="0 0 20 20">
                    <path fill-rule="evenodd" d="M10 9a3 3 0 100-6 3 3 0 000 6zm-7 9a7 7 0 1114 0H3z" clip-rule="evenodd"/>
                </svg>
                <span>{source}</span>
            </div>
            {/if}
            <!-- Commit Hash -->
            {#if commitHash()}
            <div class="flex items-center gap-1.5">
                <svg class="w-3 h-3" fill="currentColor" viewBox="0 0 20 20">
                    <path fill-rule="evenodd" d="M7.293 14.707a1 1 0 010-1.414L10.586 10 7.293 6.707a1 1 0 011.414-1.414l4 4a1 1 0 010 1.414l-4 4a1 1 0 01-1.414 0z" clip-rule="evenodd"/>
                </svg>
                <span class="font-mono">{commitHash()}</span>
            </div>
            {/if}
            <!-- Duration -->
            <div class="flex items-center gap-1.5">
                <svg class="w-3 h-3" fill="currentColor" viewBox="0 0 20 20">
                    <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm1-12a1 1 0 10-2 0v4a1 1 0 00.293.707l2.828 2.829a1 1 0 101.415-1.415L11 9.586V6z" clip-rule="evenodd"/>
                </svg>
                <span>{duration}</span>
            </div>
        </div>
    </div>

    <!-- Timestamp -->
    <div class="flex-shrink-0 text-xs text-gray-500 font-mono">
        {formatDate(lastExecuted)}
    </div>

    <!-- Details Button -->
    <button
        onclick={gotoWeb}
        class="
            flex-shrink-0
            flex items-center gap-1.5
            text-xs text-gray-600
            hover:text-gray-900
            transition-colors
        "
    >
        <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14"/>
        </svg>
        Details
    </button>
</div>
