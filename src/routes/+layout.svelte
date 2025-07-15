<script lang="ts">
    import { onMount } from 'svelte';
    import { initialize } from '../lib/api';
    import UpdateDialog from '../components/UpdateDialog.svelte';

    onMount(() => {
        initialize();
    });
</script>

<UpdateDialog />
<slot />
