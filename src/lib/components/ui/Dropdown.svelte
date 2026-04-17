<script lang="ts">
    import { fade, scale } from 'svelte/transition';

    interface Option {
        value: string;
        label: string;
        disabled?: boolean;
        description?: string;
        icon?: string;
    }

    let {
        id = undefined,
        value = $bindable(''),
        options = [],
        placeholder = 'Select an option',
        disabled = false,
        class: className = '',
        onchange = undefined
    }: {
        id?: string;
        value?: string;
        options: Option[];
        placeholder?: string;
        disabled?: boolean;
        class?: string;
        onchange?: () => void;
    } = $props();

    let open = $state(false);
    let buttonRef = $state<HTMLButtonElement | null>(null);

    let selectedOption = $derived(options.find(o => o.value === value));

    function toggle() {
        if (!disabled) {
            open = !open;
        }
    }

    function select(option: Option) {
        if (!option.disabled) {
            value = option.value;
            open = false;
            onchange?.();
        }
    }

    function handleKeydown(e: KeyboardEvent) {
        if (e.key === 'Escape') {
            open = false;
            buttonRef?.focus();
        } else if (e.key === 'ArrowDown' && !open) {
            open = true;
        }
    }

    function handleClickOutside(e: MouseEvent) {
        const target = e.target as HTMLElement;
        if (!target.closest('.dropdown-container')) {
            open = false;
        }
    }

    $effect(() => {
        if (open) {
            document.addEventListener('click', handleClickOutside);
            return () => document.removeEventListener('click', handleClickOutside);
        }
        return undefined;
    });
</script>

<div class="dropdown-container relative {className}" {id}>
    <button
        bind:this={buttonRef}
        type="button"
        onclick={toggle}
        onkeydown={handleKeydown}
        {disabled}
        class="
            w-full flex items-center justify-between gap-2
            bg-white border border-gray-300 text-gray-900 text-sm rounded-lg
            p-3 transition-all duration-200 shadow-sm
            hover:border-gray-400 focus:ring-2 focus:ring-orange-500 focus:border-orange-500
            disabled:opacity-50 disabled:cursor-not-allowed
            {open ? 'ring-2 ring-orange-500 border-orange-500' : ''}
        "
        aria-haspopup="listbox"
        aria-expanded={open}
    >
        <span class="flex items-center gap-2 truncate">
            {#if selectedOption}
                {#if selectedOption.icon}
                    <span class="text-base">{selectedOption.icon}</span>
                {/if}
                <span class="font-medium">{selectedOption.label}</span>
                {#if selectedOption.disabled}
                    <span class="text-xs bg-amber-100 text-amber-700 px-1.5 py-0.5 rounded">Coming Soon</span>
                {/if}
            {:else}
                <span class="text-gray-400">{placeholder}</span>
            {/if}
        </span>
        <svg
            class="w-4 h-4 text-gray-500 transition-transform duration-200 {open ? 'rotate-180' : ''}"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
        >
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
        </svg>
    </button>

    {#if open}
        <div
            class="absolute z-50 w-full mt-1 bg-white border border-gray-200 rounded-lg shadow-lg overflow-hidden"
            transition:scale={{ duration: 150, start: 0.95 }}
            role="listbox"
        >
            <ul class="py-1 max-h-60 overflow-auto">
                {#each options as option (option.value)}
                    <li>
                        <button
                            type="button"
                            onclick={() => select(option)}
                            disabled={option.disabled}
                            class="
                                w-full text-left px-3 py-2.5 flex items-center gap-3
                                transition-colors duration-150
                                {option.disabled ? 'opacity-50 cursor-not-allowed' : 'hover:bg-orange-50 cursor-pointer'}
                                {value === option.value ? 'bg-orange-50 text-orange-700' : 'text-gray-700'}
                            "
                            role="option"
                            aria-selected={value === option.value}
                        >
                            {#if option.icon}
                                <span class="text-lg">{option.icon}</span>
                            {/if}
                            <div class="flex-1 min-w-0">
                                <div class="flex items-center gap-2">
                                    <span class="font-medium truncate">{option.label}</span>
                                    {#if option.disabled}
                                        <span class="text-xs bg-amber-100 text-amber-700 px-1.5 py-0.5 rounded shrink-0">Coming Soon</span>
                                    {/if}
                                </div>
                                {#if option.description}
                                    <p class="text-xs text-gray-500 truncate mt-0.5">{option.description}</p>
                                {/if}
                            </div>
                            {#if value === option.value}
                                <svg class="w-4 h-4 text-orange-600 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                                </svg>
                            {/if}
                        </button>
                    </li>
                {/each}
            </ul>
        </div>
    {/if}
</div>
