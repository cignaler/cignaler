<script lang="ts">
  import { fade, scale } from 'svelte/transition';
  import { onMount } from 'svelte';

  let {
    title,
    open = $bindable(false),
    autoclose = false,
    size = 'md',
    class: className = '',
    children,
    footer
  }: {
    title: string;
    open?: boolean;
    autoclose?: boolean;
    size?: 'sm' | 'md' | 'lg' | 'xl';
    class?: string;
    children?: any;
    footer?: any;
  } = $props();

  const sizeClasses = {
    sm: 'max-w-sm',
    md: 'max-w-md',
    lg: 'max-w-lg',
    xl: 'max-w-xl'
  };

  let modalElement: HTMLDivElement;

  function handleBackdropClick(event: MouseEvent) {
    if (autoclose && event.target === event.currentTarget) {
      open = false;
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape' && autoclose && open) {
      open = false;
    }
  }

  function trapFocus(node: HTMLElement) {
    const focusableElements = node.querySelectorAll<HTMLElement>(
      'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'
    );
    const firstElement = focusableElements[0];
    const lastElement = focusableElements[focusableElements.length - 1];

    function handleTab(e: KeyboardEvent) {
      if (e.key !== 'Tab') return;

      if (e.shiftKey) {
        if (document.activeElement === firstElement) {
          e.preventDefault();
          lastElement?.focus();
        }
      } else {
        if (document.activeElement === lastElement) {
          e.preventDefault();
          firstElement?.focus();
        }
      }
    }

    node.addEventListener('keydown', handleTab);

    // Focus first element when modal opens
    setTimeout(() => firstElement?.focus(), 0);

    return {
      destroy() {
        node.removeEventListener('keydown', handleTab);
      }
    };
  }

  $effect(() => {
    if (open) {
      document.addEventListener('keydown', handleKeydown);
      return () => {
        document.removeEventListener('keydown', handleKeydown);
      };
    }
    return undefined;
  });
</script>

{#if open}
  <div
    class="fixed inset-0 z-50 flex items-center justify-center p-4"
    onclick={handleBackdropClick}
    role="presentation"
    transition:fade={{ duration: 200 }}
  >
    <!-- Backdrop -->
    <div class="fixed inset-0 bg-gray-900/60 backdrop-blur-sm dark:bg-gray-900/80"></div>

    <!-- Modal -->
    <div
      bind:this={modalElement}
      use:trapFocus
      class="relative bg-white/95 backdrop-blur-md rounded-2xl shadow-2xl dark:bg-gray-700 w-full {sizeClasses[size]} {className} border border-gray-200/50"
      transition:scale={{ duration: 200, start: 0.95 }}
      role="dialog"
      aria-modal="true"
      aria-labelledby="modal-title"
    >
      <!-- Header -->
      <div class="flex items-start justify-between p-6 border-b border-gray-200/60 rounded-t dark:border-gray-600">
        <h3 id="modal-title" class="text-2xl font-bold text-gray-900 dark:text-white">
          {title}
        </h3>
        <button
          type="button"
          class="text-gray-400 bg-transparent hover:bg-gray-200 hover:text-gray-900 rounded-lg text-sm w-8 h-8 ml-auto inline-flex justify-center items-center dark:hover:bg-gray-600 dark:hover:text-white"
          onclick={() => { if (autoclose) open = false; }}
          aria-label="Close modal"
        >
          <svg class="w-3 h-3" aria-hidden="true" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 14 14">
            <path stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="m1 1 6 6m0 0 6 6M7 7l6-6M7 7l-6 6"/>
          </svg>
        </button>
      </div>

      <!-- Body -->
      <div class="p-6 space-y-6">
        {@render children?.()}
      </div>

      <!-- Footer -->
      {#if footer}
        <div class="flex items-center p-6 space-x-2 border-t border-gray-200 rounded-b dark:border-gray-600">
          {@render footer?.()}
        </div>
      {/if}
    </div>
  </div>
{/if}
