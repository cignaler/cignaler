<script lang="ts">
    import { open } from '@tauri-apps/plugin-shell';

    let {
        webUrl,
        name,
        state,
        lastExecuted = null,
        updatedAt = null,
        sha = null,
        source = null
    }: {
        webUrl: string;
        name: string;
        state: string;
        lastExecuted?: string | null;
        updatedAt?: string | null;
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

    function formatTimeAgo(date: string | null): string {
        if (!date) return "N/A";
        const now = Date.now();
        const then = new Date(date).getTime();
        const diffMs = now - then;

        if (diffMs < 0) return "just now";

        const seconds = Math.floor(diffMs / 1000);
        const minutes = Math.floor(seconds / 60);
        const hours = Math.floor(minutes / 60);
        const days = Math.floor(hours / 24);

        if (seconds < 60) return `${seconds}s ago`;
        if (minutes < 60) return `${minutes}m ago`;
        if (hours < 24) return `${hours}h ago`;
        if (days === 1) return "Yesterday";
        return `${days}d ago`;
    }

    function formatElapsed(date: string | null): string {
        if (!date) return "N/A";
        const now = Date.now();
        const then = new Date(date).getTime();
        const diffMs = now - then;

        if (diffMs < 0) return "just started";

        const seconds = Math.floor(diffMs / 1000);
        const minutes = Math.floor(seconds / 60);
        const hours = Math.floor(minutes / 60);

        if (seconds < 60) return `running for ${seconds}s`;
        if (minutes < 60) return `running for ${minutes}m`;
        return `running for ${hours}h ${minutes % 60}m`;
    }

    // Commit hash from SHA or extract from ref name
    let commitHash = $derived.by(() => {
        if (sha) return sha.substring(0, 7);
        // Fallback: check if ref looks like a SHA
        if (name && name.match(/^[a-f0-9]{40}$/i)) return name.substring(0, 7);
        return null;
    });

    // Time info for clock icon
    let isRunning = $derived(state === "running" || state === "pending");
    let duration = $derived(
        isRunning
            ? formatElapsed(lastExecuted)
            : formatTimeAgo(updatedAt || lastExecuted)
    );

    // Dynamic border color based on status
    let borderColorClass = $derived(
        state === "success" ? "border-l-green-500" :
        state === "failed" ? "border-l-red-500" :
        state === "blocked" || state === "manual" ? "border-l-gray-400" :
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
    <div class="flex-shrink-0 w-[100px]">
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
        {:else if state === "blocked" || state === "manual"}
            <div class="flex items-center gap-1.5 bg-gray-100 text-gray-500 px-2.5 py-1 rounded text-xs font-medium uppercase">
                <svg class="w-3 h-3" fill="currentColor" viewBox="0 0 20 20">
                    <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-8-3a1 1 0 00-.867.5 1 1 0 11-1.731-1A3 3 0 0113 8a3.001 3.001 0 01-2 2.83V11a1 1 0 11-2 0v-1a1 1 0 011-1 1 1 0 100-2zm0 8a1 1 0 100-2 1 1 0 000 2z" clip-rule="evenodd"/>
                </svg>
                Blocked
            </div>
        {:else if state === "skipped" || state === "canceled"}
            <div class="flex items-center gap-1.5 bg-gray-100 text-gray-400 px-2.5 py-1 rounded text-xs font-medium uppercase">
                <svg class="w-3 h-3" fill="currentColor" viewBox="0 0 20 20">
                    <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM7 9a1 1 0 000 2h6a1 1 0 100-2H7z" clip-rule="evenodd"/>
                </svg>
                {state === "skipped" ? "Skipped" : "Canceled"}
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
                    <path fill-rule="evenodd" d="M11.3 1.046A1 1 0 0112 2v5h4a1 1 0 01.82 1.573l-7 10A1 1 0 018 18v-5H4a1 1 0 01-.82-1.573l7-10a1 1 0 011.12-.38z" clip-rule="evenodd"/>
                </svg>
                <span>{source}</span>
            </div>
            {/if}
            <!-- Commit Hash -->
            {#if commitHash}
            <div class="flex items-center gap-1.5">
                <svg class="w-3 h-3" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                    <circle cx="12" cy="12" r="4"/><line x1="1.05" y1="12" x2="8" y2="12"/><line x1="16" y1="12" x2="22.95" y2="12"/>
                </svg>
                <span class="font-mono">{commitHash}</span>
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
