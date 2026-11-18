<script lang="ts">
  import '@fontsource-variable/atkinson-hyperlegible-next';
  import '#lib/styles/global.css';

  import type { Snippet } from 'svelte';

  const { children }: { children: Snippet } = $props();
</script>

<main>
  {@render children()}
</main>
