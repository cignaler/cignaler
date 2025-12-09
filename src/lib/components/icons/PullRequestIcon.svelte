<script lang="ts">
  let {
    class: className = '',
    size = 5
  }: {
    class?: string;
    size?: number;
  } = $props();

  const sizeClasses: Record<number, string> = {
    3: 'w-3 h-3',
    4: 'w-4 h-4',
    5: 'w-5 h-5',
    6: 'w-6 h-6',
    8: 'w-8 h-8'
  };

  const sizeClass = sizeClasses[size] || 'w-5 h-5';
  const classes = `${sizeClass} ${className}`.trim();
</script>

<svg
  class={classes}
  viewBox="0 0 24 24"
  fill="none"
  stroke="currentColor"
  stroke-width="2"
  stroke-linecap="round"
  stroke-linejoin="round"
  xmlns="http://www.w3.org/2000/svg"
>
  <circle cx="18" cy="18" r="3"/>
  <circle cx="6" cy="6" r="3"/>
  <path d="M13 6h3a2 2 0 0 1 2 2v7"/>
  <line x1="6" y1="9" x2="6" y2="21"/>
</svg>
