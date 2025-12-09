<script lang="ts">
  let {
    color = 'blue',
    size = 'md',
    class: className = ''
  }: {
    color?: 'blue' | 'red' | 'green' | 'yellow' | 'gray';
    size?: 'xs' | 'sm' | 'md' | 'lg';
    class?: string;
  } = $props();

  const sizeClasses = {
    xs: 'w-2 h-2',
    sm: 'w-2.5 h-2.5',
    md: 'w-3 h-3',
    lg: 'w-3.5 h-3.5'
  };

  const colorClasses = {
    blue: 'bg-blue-600 shadow-lg shadow-blue-500/40',
    red: 'bg-red-600 shadow-lg shadow-red-500/40 animate-pulse',
    green: 'bg-green-600 shadow-lg shadow-green-500/40',
    yellow: 'bg-yellow-400 shadow-lg shadow-yellow-400/40 animate-pulse',
    gray: 'bg-gray-400 shadow-sm'
  };

  const baseClasses = 'inline-flex rounded-full transition-all duration-300';
  const classes = `${baseClasses} ${sizeClasses[size]} ${colorClasses[color]} ${className}`.trim();
</script>

<span class={classes}></span>
