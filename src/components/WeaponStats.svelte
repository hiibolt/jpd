<script lang="ts">
    import { invoke } from '@tauri-apps/api/core';
    import { games } from '../stores/state';
    import StatField from './StatField.svelte';
    import { setWeaponConfig } from '$lib/api';

    export let weaponId: string;
    export let config: any;
    export let type: string;

</script>


<div class="stats-group">
    {#if config.description}
        <p class="description">{config.description}</p>
    {/if}

    {#if type === 'SingleFire'}
        <StatField label="Trigger Delay (ms)" value={config.trigger_delay_ms} type="number" onChange={(v) => setWeaponConfig(weaponId, 'trigger_delay_ms', v)} />
        <StatField label="Recoil Completion (ms)" value={config.recoil_completion_ms} type="number" onChange={(v) => setWeaponConfig(weaponId, 'recoil_completion_ms', v)} />
        <StatField label="Release Delay (ms)" value={config.release_delay_ms} type="number" onChange={(v) => setWeaponConfig(weaponId, 'release_delay_ms', v)} />
        <StatField label="DX" value={config.dx} type="number" onChange={(v) => setWeaponConfig(weaponId, 'dx', v)} />
        <StatField label="DY" value={config.dy} type="number" onChange={(v) => setWeaponConfig(weaponId, 'dy', v)} />

        <button class="autofire-toggle" on:click={() =>  setWeaponConfig(weaponId, 'autofire', !config.autofire)}>
            <b>{config.autofire ? 'Disable Autofire' : 'Enable Autofire'}</b>
        </button>

        <button class="recoil-toggle" on:click={() =>  setWeaponConfig(weaponId, 'enabled', !config.enabled)}>
            <b>{config.enabled ? 'Disable Recoil Control' : 'Enable Recoil Control'}</b>
        </button>

    {:else if type === 'SingleShot'}
        <StatField label="Recoil Completion (ms)" value={config.recoil_completion_ms} type="number" onChange={(v) => setWeaponConfig(weaponId, 'recoil_completion_ms', v)} />
        <StatField label="DX" value={config.dx} type="number" onChange={(v) => setWeaponConfig(weaponId, 'dx', v)} />
        <StatField label="DY" value={config.dy} type="number" onChange={(v) => setWeaponConfig(weaponId, 'dy', v)} />

        <button class="recoil-toggle" on:click={() =>  setWeaponConfig(weaponId, 'enabled', !config.enabled)}>
            <b>{config.enabled ? 'Disable Recoil Control' : 'Enable Recoil Control'}</b>
        </button>

    {:else if type === 'FullAutoStandard'}
        <StatField label="RPM" value={config.rpm} type="number" onChange={(v) => setWeaponConfig(weaponId, 'rpm', v)} />
        <StatField label="First Shot Scale" value={config.first_shot_scale} type="number" onChange={(v) => setWeaponConfig(weaponId, 'first_shot_scale', v)} />
        <StatField label="Exponential Factor" value={config.exponential_factor} type="number" onChange={(v) => setWeaponConfig(weaponId, 'exponential_factor', v)} />
        <StatField label="DX" value={config.dx} type="number" onChange={(v) => setWeaponConfig(weaponId, 'dx', v)} />
        <StatField label="DY" value={config.dy} type="number" onChange={(v) => setWeaponConfig(weaponId, 'dy', v)} />

        <button class="recoil-toggle" on:click={() =>  setWeaponConfig(weaponId, 'enabled', !config.enabled)}>
            <b>{config.enabled ? 'Disable Recoil Control' : 'Enable Recoil Control'}</b>
        </button>
    
    {:else if type === 'None'}
        <p class="no-recoil-info">This weapon performs no recoil control.</p>
        
        <button class="recoil-toggle" on:click={() =>  setWeaponConfig(weaponId, 'enabled', !config.enabled)}>
            <b>{config.enabled ? 'Disable Recoil Control' : 'Enable Recoil Control'}</b>
        </button>
    {/if}
</div>

<style>
    .autofire-toggle,
    .recoil-toggle {
        background: var(--button-bg);
        color: var(--button-fg);
        width: 100%;
        padding: 0.5rem 1rem;
        border: 1px solid var(--border-color, #ccc);
        border-radius: 8px;
        cursor: pointer;
        transition: background 0.2s, color 0.2s;
        margin-top: 0.5rem;
    }
    .stats-group {
        margin-top: 0.5rem;
        padding-left: 1rem;
        font-size: 0.9em;
    }
    .no-recoil-info {
        color: var(--text-muted, #888);
        font-style: italic;
        margin: 0.5rem 0;
    }
</style>
