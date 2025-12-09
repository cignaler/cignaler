<script lang="ts">
  let {
    type = 'text',
    id = undefined,
    placeholder = '',
    value = $bindable(''),
    disabled = false,
    required = false,
    list = undefined,
    class: className = ''
  }: {
    type?: 'text' | 'password' | 'email' | 'url' | 'number';
    id?: string;
    placeholder?: string;
    value?: string;
    disabled?: boolean;
    required?: boolean;
    list?: string;
    class?: string;
  } = $props();

  const baseClasses = 'bg-white border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500 block w-full p-3 transition-all duration-200 shadow-sm hover:border-gray-400 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-primary-500 dark:focus:border-primary-500';
  const disabledClasses = disabled ? 'opacity-50 cursor-not-allowed' : '';
  const classes = `${baseClasses} ${disabledClasses} ${className}`.trim();
</script>

<input
  {type}
  {id}
  {placeholder}
  bind:value
  {disabled}
  {required}
  {list}
  class={classes}
/>
