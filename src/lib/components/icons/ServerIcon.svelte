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

  let sizeClass = $derived(sizeClasses[size] || 'w-5 h-5');
  let classes = $derived(`${sizeClass} ${className}`.trim());
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
  <rect width="20" height="8" x="2" y="2" rx="2" ry="2"/>
  <rect width="20" height="8" x="2" y="14" rx="2" ry="2"/>
  <line x1="6" x2="6.01" y1="6" y2="6"/>
  <line x1="6" x2="6.01" y1="18" y2="18"/>
  <path d="m16 6 2 2-2 2"/>
  <path d="M10 18h4"/>
</svg>
