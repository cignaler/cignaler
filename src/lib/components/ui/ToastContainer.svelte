<script lang="ts">
    import { toastState, dismissToast } from "../../stores/toast.svelte";
    import { fly } from "svelte/transition";

    const typeStyles = {
        success: "bg-green-100 text-green-800 border-green-200",
        error: "bg-red-100 text-red-800 border-red-200",
        warning: "bg-yellow-100 text-yellow-800 border-yellow-200",
        info: "bg-blue-100 text-blue-800 border-blue-200"
    };

    const typeIcons = {
        success: "M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z",
        error: "M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z",
        warning: "M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z",
        info: "M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
    };
</script>

<div class="fixed bottom-4 right-4 z-50 flex flex-col gap-2 max-w-sm">
    {#each toastState.toasts as toast (toast.id)}
        <div
            class="flex items-start gap-3 px-4 py-3 rounded-lg shadow-lg border {typeStyles[toast.type]}"
            transition:fly={{ x: 100, duration: 300 }}
        >
            <svg class="w-5 h-5 flex-shrink-0 mt-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d={typeIcons[toast.type]} />
            </svg>
            <span class="flex-1 text-sm">{toast.message}</span>
            <button
                onclick={() => dismissToast(toast.id)}
                class="flex-shrink-0 p-0.5 hover:bg-black/10 rounded transition-colors"
                aria-label="Dismiss notification"
            >
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
                </svg>
            </button>
        </div>
    {/each}
</div>
