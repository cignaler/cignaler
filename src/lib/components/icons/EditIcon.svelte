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
  <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/>
  <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/>
</svg>
