<script lang="ts">
    import { invoke } from '@tauri-apps/api/core';
    import { games } from '../stores/state';

    export let id: string;
    export let config: any;
    export let type: string;

    async function setAutofire ( enabled: boolean, weapon: string ) {
        await invoke('set_autofire', { enabled, weapon });

        // Reload all weapons to reflect changes
        const loadGames = await invoke('get_games');
        games.set(loadGames as any);
    }
</script>

<div class="stats-group">
    {#if type === 'SingleFire'}
        <p>Trigger Delay: {config.trigger_delay_ms} ms</p>
        <p>Recoil Completion: {config.recoil_completion_ms} ms</p>
        <p>Release Delay: {config.release_delay_ms} ms</p>
        <p>DX/DY: {config.dx}/{config.dy}</p>
        <p>Mag Size: {config.mag_size}</p>
        <button class="autofire-toggle" onclick={() => setAutofire(!config.autofire, id)}>
            {config.autofire ? 'Disable Autofire' : 'Enable Autofire'}
        </button>
    {:else if type === 'FullAutoStandard'}
        <p>RPM: {config.rpm}</p>
        <p>First Shot Scale: {config.first_shot_scale}</p>
        <p>Exponential Factor: {config.exponential_factor}</p>
        <p>DX/DY: {config.dx}/{config.dy}</p>
        <p>Mag Size: {config.mag_size}</p>
    {/if}
</div>

<style>
    .autofire-toggle {
        background: var(--button-bg);
        color: var(--button-fg);
        border: white 1px solid;
        width: 100%;
        padding: 0.5rem 1rem;
        border-radius: 4px;
        cursor: pointer;
        transition: background 0.2s, color 0.2s;
    }
    .stats-group {
        margin-top: 0.5rem;
        padding-left: 1rem;
        font-size: 0.9em;
    }
</style>
