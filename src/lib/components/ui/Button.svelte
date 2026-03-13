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
    primary: 'bg-gradient-to-r from-primary-600 to-primary-500 text-white hover:from-primary-700 hover:to-primary-600 focus:ring-primary-300 shadow-lg shadow-primary-500/30 hover:shadow-xl hover:shadow-primary-500/40 dark:from-primary-600 dark:to-primary-500 dark:hover:from-primary-700 dark:hover:to-primary-600 dark:focus:ring-primary-800',
    secondary: 'bg-gradient-to-r from-secondary-600 to-secondary-500 text-white hover:from-secondary-700 hover:to-secondary-600 focus:ring-secondary-300 shadow-lg shadow-secondary-500/30 hover:shadow-xl hover:shadow-secondary-500/40 dark:from-secondary-600 dark:to-secondary-500 dark:hover:from-secondary-700 dark:hover:to-secondary-600 dark:focus:ring-secondary-800',
    alternative: 'bg-gray-200 text-gray-900 hover:bg-gray-300 focus:ring-gray-100 shadow-sm hover:shadow-md dark:bg-gray-700 dark:text-white dark:hover:bg-gray-600 dark:focus:ring-gray-700',
    light: 'bg-white text-gray-900 border border-gray-300 hover:bg-gray-50 hover:border-gray-400 focus:ring-gray-200 shadow-sm hover:shadow-md dark:bg-gray-800 dark:text-white dark:border-gray-600 dark:hover:bg-gray-700 dark:focus:ring-gray-700'
  };

  const outlineColorClasses = {
    primary: 'text-primary-700 border border-primary-700 hover:bg-primary-800 hover:text-white focus:ring-primary-300 dark:border-primary-500 dark:text-primary-500 dark:hover:bg-primary-600 dark:hover:text-white dark:focus:ring-primary-800',
    secondary: 'text-secondary-700 border border-secondary-700 hover:bg-secondary-800 hover:text-white focus:ring-secondary-300 dark:border-secondary-500 dark:text-secondary-500 dark:hover:bg-secondary-600 dark:hover:text-white dark:focus:ring-secondary-800',
    alternative: 'text-gray-900 border border-gray-300 hover:bg-gray-100 focus:ring-gray-200 dark:text-gray-400 dark:border-gray-600 dark:hover:bg-gray-700 dark:hover:text-white dark:focus:ring-gray-700',
    light: 'text-gray-900 border border-gray-300 hover:bg-gray-100 focus:ring-gray-200 dark:text-gray-400 dark:border-gray-600 dark:hover:bg-gray-700 dark:hover:text-white dark:focus:ring-gray-700'
  };

  const baseClasses = 'inline-flex items-center justify-center font-medium rounded-lg transition-colors focus:outline-none focus:ring-4';
  let disabledClasses = $derived(disabled ? 'opacity-50 cursor-not-allowed' : '');
  let colorClass = $derived(outline ? outlineColorClasses[color] : solidColorClasses[color]);
  let classes = $derived(`${baseClasses} ${sizeClasses[size]} ${colorClass} ${disabledClasses} ${className}`.trim());
</script>

<button
  class={classes}
  {disabled}
  {onclick}
  type="button"
>
  {@render children?.()}
</button>
