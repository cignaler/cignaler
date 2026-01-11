<script lang="ts">
    import Modal from './Modal.svelte';
    import Button from './Button.svelte';

    let {
        open = $bindable(false),
        title = "Confirm",
        message,
        confirmText = "Confirm",
        cancelText = "Cancel",
        variant = "danger" as "danger" | "warning" | "info",
        loading = false,
        onconfirm,
        oncancel
    }: {
        open?: boolean;
        title?: string;
        message: string;
        confirmText?: string;
        cancelText?: string;
        variant?: "danger" | "warning" | "info";
        loading?: boolean;
        onconfirm: () => void | Promise<void>;
        oncancel?: () => void;
    } = $props();

    const variantColors = {
        danger: "primary" as const,
        warning: "primary" as const,
        info: "primary" as const
    };

    const variantIcons = {
        danger: "M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z",
        warning: "M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z",
        info: "M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
    };

    const variantBgColors = {
        danger: "bg-red-100",
        warning: "bg-yellow-100",
        info: "bg-blue-100"
    };

    const variantIconColors = {
        danger: "text-red-600",
        warning: "text-yellow-600",
        info: "text-blue-600"
    };

    async function handleConfirm() {
        await onconfirm();
        open = false;
    }

    function handleCancel() {
        oncancel?.();
        open = false;
    }
</script>

<Modal {title} bind:open size="sm">
    {#snippet children()}
        <div class="flex gap-4">
            <div class="flex-shrink-0 w-10 h-10 rounded-full {variantBgColors[variant]} flex items-center justify-center">
                <svg class="w-6 h-6 {variantIconColors[variant]}" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d={variantIcons[variant]} />
                </svg>
            </div>
            <p class="text-gray-600 flex-1">{message}</p>
        </div>
    {/snippet}

    {#snippet footer()}
        <Button color={variantColors[variant]} onclick={handleConfirm} disabled={loading}>
            {#if loading}
                <svg class="animate-spin -ml-1 mr-2 h-4 w-4" fill="none" viewBox="0 0 24 24">
                    <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                    <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                </svg>
                Processing...
            {:else}
                {confirmText}
            {/if}
        </Button>
        <Button color="alternative" onclick={handleCancel} disabled={loading}>
            {cancelText}
        </Button>
    {/snippet}
</Modal>
