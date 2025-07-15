<script lang="ts">
    import { onMount } from 'svelte';
    import { initialize } from '../lib/api';
    import { initializeTheme } from '../lib/themeManager';
    import UpdateDialog from '../components/UpdateDialog.svelte';
    import '../lib/styles/global.css';

    onMount(() => {
        initialize();
        initializeTheme();
    });
</script>

<UpdateDialog />
<slot />
