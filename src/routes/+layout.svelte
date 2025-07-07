<script lang="ts">
    import { onMount } from 'svelte';
    import { initialize } from '../lib/api';

    onMount(() => {
        initialize();
    });
</script>

<slot />
