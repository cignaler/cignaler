<script lang="ts">
  let {
    color = 'primary',
    size = 'md',
    outline = false,
    disabled = false,
    class: className = '',
    onclick = undefined,
    children
  }: {
    color?: 'primary' | 'secondary' | 'alternative' | 'light';
    size?: 'xs' | 'sm' | 'md' | 'lg';
    outline?: boolean;
    disabled?: boolean;
    class?: string;
    onclick?: () => void;
    children?: any;
  } = $props();

  const sizeClasses = {
    xs: 'px-3 py-2 text-xs',
    sm: 'px-4 py-2 text-sm',
    md: 'px-5 py-2.5 text-sm',
    lg: 'px-5 py-3 text-base'
  };

  const solidColorClasses = {
    primary: 'bg-primary-600 text-white hover:bg-primary-700 focus:ring-primary-300 dark:bg-primary-600 dark:hover:bg-primary-700 dark:focus:ring-primary-800',
    secondary: 'bg-secondary-600 text-white hover:bg-secondary-700 focus:ring-secondary-300 dark:bg-secondary-600 dark:hover:bg-secondary-700 dark:focus:ring-secondary-800',
    alternative: 'bg-gray-200 text-gray-900 hover:bg-gray-300 focus:ring-gray-100 dark:bg-gray-700 dark:text-white dark:hover:bg-gray-600 dark:focus:ring-gray-700',
    light: 'bg-white text-gray-900 border border-gray-300 hover:bg-gray-100 focus:ring-gray-200 dark:bg-gray-800 dark:text-white dark:border-gray-600 dark:hover:bg-gray-700 dark:focus:ring-gray-700'
  };

  const outlineColorClasses = {
    primary: 'text-primary-700 border border-primary-700 hover:bg-primary-800 hover:text-white focus:ring-primary-300 dark:border-primary-500 dark:text-primary-500 dark:hover:bg-primary-600 dark:hover:text-white dark:focus:ring-primary-800',
    secondary: 'text-secondary-700 border border-secondary-700 hover:bg-secondary-800 hover:text-white focus:ring-secondary-300 dark:border-secondary-500 dark:text-secondary-500 dark:hover:bg-secondary-600 dark:hover:text-white dark:focus:ring-secondary-800',
    alternative: 'text-gray-900 border border-gray-300 hover:bg-gray-100 focus:ring-gray-200 dark:text-gray-400 dark:border-gray-600 dark:hover:bg-gray-700 dark:hover:text-white dark:focus:ring-gray-700',
    light: 'text-gray-900 border border-gray-300 hover:bg-gray-100 focus:ring-gray-200 dark:text-gray-400 dark:border-gray-600 dark:hover:bg-gray-700 dark:hover:text-white dark:focus:ring-gray-700'
  };

  const baseClasses = 'inline-flex items-center justify-center font-medium rounded-lg transition-colors focus:outline-none focus:ring-4';
  const disabledClasses = disabled ? 'opacity-50 cursor-not-allowed' : '';
  const colorClass = outline ? outlineColorClasses[color] : solidColorClasses[color];
  const classes = `${baseClasses} ${sizeClasses[size]} ${colorClass} ${disabledClasses} ${className}`.trim();
</script>

<button
  class={classes}
  {disabled}
  {onclick}
  type="button"
>
  {@render children?.()}
</button>
