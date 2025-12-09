<script lang="ts">
  let {
    color = 'blue',
    rounded = false,
    class: className = '',
    children
  }: {
    color?: 'blue' | 'red' | 'green' | 'yellow' | 'gray';
    rounded?: boolean;
    class?: string;
    children?: any;
  } = $props();

  const colorClasses = {
    blue: 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-300',
    red: 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-300',
    green: 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-300',
    yellow: 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-300',
    gray: 'bg-gray-100 text-gray-800 dark:bg-gray-700 dark:text-gray-300'
  };

  const baseClasses = 'inline-flex items-center font-medium px-2.5 py-0.5 text-xs';
  const roundedClass = rounded ? 'rounded-full' : 'rounded';
  const classes = `${baseClasses} ${roundedClass} ${colorClasses[color]} ${className}`.trim();
</script>

<span class={classes}>
  {@render children?.()}
</span>
