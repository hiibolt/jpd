<script lang="ts">
    import { invoke } from '@tauri-apps/api/core';
    import { games } from '../stores/state';
    import StatField from './StatField.svelte';

    export let weaponId: string;
    export let config: any;
    export let type: string;

    async function updateField ( weaponId: string, field: string, newValue: any ) {
        $games = await invoke('set_weapon_config', { weaponId, field, newValue });
    }
</script>


<div class="stats-group">
    {#if config.description}
        <p class="description">{config.description}</p>
    {/if}

    {#if type === 'SingleFire'}
        <StatField label="Trigger Delay (ms)" value={config.trigger_delay_ms} type="number" onChange={(v) => updateField(weaponId, 'trigger_delay_ms', v)} />
        <StatField label="Recoil Completion (ms)" value={config.recoil_completion_ms} type="number" onChange={(v) => updateField(weaponId, 'recoil_completion_ms', v)} />
        <StatField label="Release Delay (ms)" value={config.release_delay_ms} type="number" onChange={(v) => updateField(weaponId, 'release_delay_ms', v)} />
        <StatField label="DX" value={config.dx} type="number" onChange={(v) => updateField(weaponId, 'dx', v)} />
        <StatField label="DY" value={config.dy} type="number" onChange={(v) => updateField(weaponId, 'dy', v)} />
        <StatField label="Mag Size" value={config.mag_size} type="number" onChange={(v) => updateField(weaponId, 'mag_size', v)} />

        <button class="autofire-toggle" on:click={() =>  updateField(weaponId, 'autofire', !config.autofire)}>
            <b>{config.autofire ? 'Disable Autofire' : 'Enable Autofire'}</b>
        </button>

    {:else if type === 'FullAutoStandard'}
        <StatField label="RPM" value={config.rpm} type="number" onChange={(v) => updateField(weaponId, 'rpm', v)} />
        <StatField label="First Shot Scale" value={config.first_shot_scale} type="number" onChange={(v) => updateField(weaponId, 'first_shot_scale', v)} />
        <StatField label="Exponential Factor" value={config.exponential_factor} type="number" onChange={(v) => updateField(weaponId, 'exponential_factor', v)} />
        <StatField label="DX" value={config.dx} type="number" onChange={(v) => updateField(weaponId, 'dx', v)} />
        <StatField label="DY" value={config.dy} type="number" onChange={(v) => updateField(weaponId, 'dy', v)} />
        <StatField label="Mag Size" value={config.mag_size} type="number" onChange={(v) => updateField(weaponId, 'mag_size', v)} />
    {/if}
</div>

<style>
    .autofire-toggle {
        background: var(--button-bg);
        color: var(--button-fg);
        width: 100%;
        padding: 0.5rem 1rem;
        border: 1px solid var(--border-color, #ccc);
        border-radius: 8px;
        cursor: pointer;
        transition: background 0.2s, color 0.2s;
    }
    .stats-group {
        margin-top: 0.5rem;
        padding-left: 1rem;
        font-size: 0.9em;
    }
</style>
