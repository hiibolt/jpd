<script lang="ts">
    import MainLayout from '../layouts/MainLayout.svelte';
    import { onMount } from 'svelte';
    import { initialize } from '../lib/api';

    onMount(() => {
      initialize();
    });
</script>

<MainLayout />

<style>
  
</style>