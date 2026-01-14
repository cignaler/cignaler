<script lang="ts">
  let {
    for: htmlFor = undefined,
    class: className = '',
    children
  }: {
    for?: string;
    class?: string;
    children?: any;
  } = $props();

  const baseClasses = 'block text-sm font-medium text-gray-900 dark:text-white';
  let classes = $derived(`${baseClasses} ${className}`.trim());
</script>

<label for={htmlFor} class={classes}>
  {@render children?.()}
</label>
