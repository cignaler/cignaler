<script lang="ts">
  let {
    id = undefined,
    value = $bindable(''),
    disabled = false,
    class: className = '',
    children
  }: {
    id?: string;
    value?: string;
    disabled?: boolean;
    class?: string;
    children?: any;
  } = $props();

  const baseClasses = 'bg-white border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500 block w-full p-3 transition-all duration-200 shadow-sm hover:border-gray-400 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-primary-500 dark:focus:border-primary-500';
  const disabledClasses = disabled ? 'opacity-50 cursor-not-allowed' : '';
  const classes = `${baseClasses} ${disabledClasses} ${className}`.trim();
</script>

<select
  {id}
  bind:value
  {disabled}
  class={classes}
>
  {@render children?.()}
</select>
