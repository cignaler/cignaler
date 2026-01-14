<script lang="ts">
  let {
    color = 'blue',
    dismissible = false,
    class: className = '',
    children,
    icon
  }: {
    color?: 'red' | 'green' | 'yellow' | 'blue';
    dismissible?: boolean;
    class?: string;
    children?: any;
    icon?: any;
  } = $props();

  const colorClasses = {
    red: 'text-red-500 bg-red-100 dark:bg-red-800 dark:text-red-200',
    green: 'text-green-500 bg-green-100 dark:bg-green-800 dark:text-green-200',
    yellow: 'text-yellow-500 bg-yellow-100 dark:bg-yellow-800 dark:text-yellow-200',
    blue: 'text-blue-500 bg-blue-100 dark:bg-blue-800 dark:text-blue-200'
  };

  const baseClasses = 'flex items-center w-full max-w-xs p-4 text-gray-500 bg-white rounded-lg shadow dark:text-gray-400 dark:bg-gray-800';
  let classes = $derived(`${baseClasses} ${className}`.trim());
  let iconClasses = $derived(`inline-flex items-center justify-center flex-shrink-0 w-8 h-8 rounded-lg ${colorClasses[color]}`);
</script>

<div class={classes} role="alert">
  {#if icon}
    <div class={iconClasses}>
      {@render icon?.()}
    </div>
  {/if}
  <div class="ml-3 text-sm font-normal">
    {@render children?.()}
  </div>
</div>
