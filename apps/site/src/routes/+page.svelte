<script lang="ts">
  import Seo from '#lib/components/seo/seo.svelte';
  import { site } from '#lib/content/site.ts';
</script>

<Seo title={site.name} description={site.description} />

<h1>{site.name}</h1>

<style>
  h1 {
    color: var(--primary);
    font-size: var(--font-size-title);
    font-weight: 600;
  }
</style>
