<script lang="ts">
  let {
    type = 'text',
    id = undefined,
    placeholder = '',
    value = $bindable(''),
    disabled = false,
    required = false,
    list = undefined,
    error = '',
    helpText = '',
    class: className = '',
    oninput = undefined
  }: {
    type?: 'text' | 'password' | 'email' | 'url' | 'number';
    id?: string;
    placeholder?: string;
    value?: string;
    disabled?: boolean;
    required?: boolean;
    list?: string;
    error?: string;
    helpText?: string;
    class?: string;
    oninput?: (e: Event) => void;
  } = $props();

  // Generate IDs for accessibility
  let inputId = $derived(id || `input-${Math.random().toString(36).slice(2)}`);
  let helpId = $derived(`${inputId}-help`);
  let errorId = $derived(`${inputId}-error`);
  let ariaDescribedBy = $derived(
    [helpText ? helpId : '', error ? errorId : ''].filter(Boolean).join(' ') || undefined
  );

  const baseClasses = 'bg-white border text-gray-900 text-sm rounded-lg focus:ring-2 block w-full p-3 transition-all duration-200 shadow-sm dark:bg-gray-700 dark:placeholder-gray-400 dark:text-white';
  let stateClasses = $derived(
    error
      ? 'border-red-500 focus:ring-red-500 focus:border-red-500 dark:border-red-500'
      : 'border-gray-300 hover:border-gray-400 focus:ring-primary-500 focus:border-primary-500 dark:border-gray-600 dark:focus:ring-primary-500 dark:focus:border-primary-500'
  );
  let disabledClasses = $derived(disabled ? 'opacity-50 cursor-not-allowed' : '');
  let classes = $derived(`${baseClasses} ${stateClasses} ${disabledClasses} ${className}`.trim());
</script>

<div class="w-full">
  <input
    {type}
    id={inputId}
    {placeholder}
    bind:value
    {disabled}
    {required}
    {list}
    {oninput}
    class={classes}
    aria-required={required ? 'true' : undefined}
    aria-invalid={error ? 'true' : undefined}
    aria-describedby={ariaDescribedBy}
  />

  {#if helpText && !error}
    <p id={helpId} class="text-xs text-gray-500 mt-1">{helpText}</p>
  {/if}

  {#if error}
    <p id={errorId} class="text-xs text-red-500 mt-1" role="alert">{error}</p>
  {/if}
</div>
