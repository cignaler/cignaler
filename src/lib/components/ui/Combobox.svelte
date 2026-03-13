<script lang="ts">
    import { scale } from 'svelte/transition';

    interface Props {
        id?: string;
        value?: string;
        options?: string[];
        placeholder?: string;
        disabled?: boolean;
        loading?: boolean;
        error?: string;
        helpText?: string;
        class?: string;
        onrefresh?: () => void;
    }

    let {
        id = undefined,
        value = $bindable(''),
        options = [],
        placeholder = 'Type to search...',
        disabled = false,
        loading = false,
        error = '',
        helpText = '',
        class: className = '',
        onrefresh
    }: Props = $props();

    let open = $state(false);
    let inputRef = $state<HTMLInputElement | null>(null);
    let listRef = $state<HTMLUListElement | null>(null);
    let highlightedIndex = $state(-1);

    // Filter options based on input value
    let filteredOptions = $derived(
        value.trim()
            ? options.filter(opt =>
                opt.toLowerCase().includes(value.toLowerCase())
            )
            : options
    );

    // IDs for accessibility
    let inputId = $derived(id || `combobox-${Math.random().toString(36).slice(2)}`);
    let listId = $derived(`${inputId}-list`);
    let helpId = $derived(`${inputId}-help`);
    let errorId = $derived(`${inputId}-error`);
    let ariaDescribedBy = $derived(
        [helpText ? helpId : '', error ? errorId : ''].filter(Boolean).join(' ') || undefined
    );

    function handleFocus() {
        if (!disabled) {
            open = true;
            highlightedIndex = -1;
        }
    }

    function handleBlur(e: FocusEvent) {
        // Delay close to allow click on option
        setTimeout(() => {
            const active = document.activeElement;
            if (!listRef?.contains(active) && active !== inputRef) {
                open = false;
                highlightedIndex = -1;
            }
        }, 150);
    }

    function handleInput() {
        open = true;
        highlightedIndex = -1;
    }

    function selectOption(option: string) {
        value = option;
        open = false;
        highlightedIndex = -1;
        inputRef?.focus();
    }

    function handleKeydown(e: KeyboardEvent) {
        if (disabled) return;

        switch (e.key) {
            case 'ArrowDown':
                e.preventDefault();
                if (!open) {
                    open = true;
                } else {
                    highlightedIndex = Math.min(highlightedIndex + 1, filteredOptions.length - 1);
                    scrollToHighlighted();
                }
                break;
            case 'ArrowUp':
                e.preventDefault();
                if (open) {
                    highlightedIndex = Math.max(highlightedIndex - 1, 0);
                    scrollToHighlighted();
                }
                break;
            case 'Enter':
                e.preventDefault();
                const selectedOption = filteredOptions[highlightedIndex];
                if (open && highlightedIndex >= 0 && selectedOption) {
                    selectOption(selectedOption);
                }
                break;
            case 'Escape':
                e.preventDefault();
                open = false;
                highlightedIndex = -1;
                break;
            case 'Tab':
                open = false;
                highlightedIndex = -1;
                break;
        }
    }

    function scrollToHighlighted() {
        if (highlightedIndex >= 0 && listRef) {
            const item = listRef.children[highlightedIndex] as HTMLElement;
            item?.scrollIntoView({ block: 'nearest' });
        }
    }

    function handleClickOutside(e: MouseEvent) {
        const target = e.target as HTMLElement;
        if (!target.closest('.combobox-container')) {
            open = false;
            highlightedIndex = -1;
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

<div class="combobox-container relative {className}">
    <div class="relative">
        <input
            bind:this={inputRef}
            type="text"
            id={inputId}
            bind:value
            {placeholder}
            {disabled}
            onfocus={handleFocus}
            onblur={handleBlur}
            oninput={handleInput}
            onkeydown={handleKeydown}
            role="combobox"
            aria-expanded={open}
            aria-controls={listId}
            aria-autocomplete="list"
            aria-activedescendant={highlightedIndex >= 0 ? `${listId}-option-${highlightedIndex}` : undefined}
            aria-describedby={ariaDescribedBy}
            aria-invalid={error ? 'true' : undefined}
            class="
                w-full bg-white border text-gray-900 text-sm rounded-lg
                block p-3 pr-20 transition-all duration-200 shadow-sm
                focus:ring-2 focus:ring-orange-500 focus:border-orange-500
                disabled:opacity-50 disabled:cursor-not-allowed
                {error ? 'border-red-500 focus:ring-red-500 focus:border-red-500' : 'border-gray-300 hover:border-gray-400'}
            "
        />

        <div class="absolute right-1 top-1/2 -translate-y-1/2 flex items-center gap-1">
            {#if loading}
                <div class="p-2">
                    <svg class="w-4 h-4 animate-spin text-gray-400" fill="none" viewBox="0 0 24 24">
                        <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                        <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                    </svg>
                </div>
            {:else if onrefresh}
                <button
                    type="button"
                    onclick={(e) => { e.stopPropagation(); onrefresh?.(); }}
                    disabled={disabled}
                    class="p-2 text-gray-400 hover:text-gray-600 hover:bg-gray-100 rounded-md transition-colors disabled:opacity-50"
                    title="Refresh options"
                    aria-label="Refresh options"
                >
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
                    </svg>
                </button>
            {/if}
            <button
                type="button"
                onclick={() => { if (!disabled) { open = !open; inputRef?.focus(); } }}
                disabled={disabled}
                class="p-2 text-gray-400 hover:text-gray-600 transition-colors disabled:opacity-50"
                tabindex="-1"
                aria-label={open ? 'Close options' : 'Open options'}
            >
                <svg
                    class="w-4 h-4 transition-transform duration-200 {open ? 'rotate-180' : ''}"
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                >
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
                </svg>
            </button>
        </div>
    </div>

    {#if open && !disabled}
        <div
            class="absolute z-50 w-full mt-1 bg-white border border-gray-200 rounded-lg shadow-lg overflow-hidden"
            transition:scale={{ duration: 150, start: 0.95 }}
        >
            {#if loading}
                <div class="p-4 text-center">
                    <div class="flex items-center justify-center gap-2 text-sm text-gray-500">
                        <svg class="w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24">
                            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                        </svg>
                        Loading options...
                    </div>
                </div>
            {:else if filteredOptions.length === 0}
                <div class="p-4 text-center text-sm text-gray-500">
                    {options.length === 0 ? 'No options available' : 'No matches found'}
                </div>
            {:else}
                <ul
                    bind:this={listRef}
                    id={listId}
                    class="py-1 max-h-60 overflow-auto"
                    role="listbox"
                >
                    {#each filteredOptions as option, index (option)}
                        <li
                            id="{listId}-option-{index}"
                            role="option"
                            aria-selected={value === option}
                        >
                            <button
                                type="button"
                                onclick={() => selectOption(option)}
                                class="
                                    w-full text-left px-3 py-2 text-sm
                                    transition-colors duration-150
                                    hover:bg-orange-50 cursor-pointer
                                    {index === highlightedIndex ? 'bg-orange-50' : ''}
                                    {value === option ? 'text-orange-700 font-medium' : 'text-gray-700'}
                                "
                            >
                                <span class="flex items-center justify-between">
                                    <span class="truncate">{option}</span>
                                    {#if value === option}
                                        <svg class="w-4 h-4 text-orange-600 shrink-0 ml-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                                        </svg>
                                    {/if}
                                </span>
                            </button>
                        </li>
                    {/each}
                </ul>
            {/if}
        </div>
    {/if}

    {#if helpText && !error}
        <p id={helpId} class="text-xs text-gray-500 mt-1">{helpText}</p>
    {/if}

    {#if error}
        <p id={errorId} class="text-xs text-red-500 mt-1" role="alert">{error}</p>
    {/if}
</div>
