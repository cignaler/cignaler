<script lang="ts">
  let {
    checked = $bindable(false),
    disabled = false,
    class: className = ''
  }: {
    checked?: boolean;
    disabled?: boolean;
    class?: string;
  } = $props();

  const baseClasses = 'w-4 h-4 text-primary-600 bg-gray-100 border-gray-300 rounded focus:ring-primary-500 dark:focus:ring-primary-600 dark:ring-offset-gray-800 focus:ring-2 dark:bg-gray-700 dark:border-gray-600';
  const disabledClasses = disabled ? 'opacity-50 cursor-not-allowed' : 'cursor-pointer';
  const classes = `${baseClasses} ${disabledClasses} ${className}`.trim();
</script>

<input
  type="checkbox"
  bind:checked
  {disabled}
  class={classes}
/>
