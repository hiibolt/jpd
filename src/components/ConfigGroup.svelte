<script lang="ts">
    import { changeSetting } from '$lib/api';
    import StatField from './StatField.svelte';

    type ConfigOption = {
        key: string;
        label: string;
        description?: string | null;
        type: 'keybind' | 'checkbox' | 'slider' | 'char';
        value: any;
    };

    export let label: string = 'Configuration Options';
    export let configOptions: ConfigOption[] = [];

    function handleUpdate(key: string, value: string | boolean | number) {
        changeSetting(key, value);
    }
</script>

<style>
    .config-group {
        max-width: 400px;
        margin: 16px auto;
    }
</style>

<div class="config-group">
    <h3>{label}</h3>
    {#each configOptions as option (option.key)}
        <StatField label={option.label} value={option.value} description={option.description} type={option.type} onChange={(v) => handleUpdate(option.key, v)} />
    {/each}
</div>
