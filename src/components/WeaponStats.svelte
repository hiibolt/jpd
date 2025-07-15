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
        <StatField label="Enable Autofire" value={config.autofire} type="checkbox" onChange={(v) => setWeaponConfig(weaponId, 'autofire', v)} />
        <StatField label="Enable Recoil Control" value={config.enabled} type="checkbox" onChange={(v) => setWeaponConfig(weaponId, 'enabled', v)} />

    {:else if type === 'SingleShot'}
        <StatField label="Recoil Completion (ms)" value={config.recoil_completion_ms} type="number" onChange={(v) => setWeaponConfig(weaponId, 'recoil_completion_ms', v)} />
        <StatField label="DX" value={config.dx} type="number" onChange={(v) => setWeaponConfig(weaponId, 'dx', v)} />
        <StatField label="DY" value={config.dy} type="number" onChange={(v) => setWeaponConfig(weaponId, 'dy', v)} />
        <StatField label="Enable Recoil Control" value={config.enabled} type="checkbox" onChange={(v) => setWeaponConfig(weaponId, 'enabled', v)} />

    {:else if type === 'FullAutoStandard'}
        <StatField label="RPM" value={config.rpm} type="number" onChange={(v) => setWeaponConfig(weaponId, 'rpm', v)} />
        <StatField label="First Shot Scale" value={config.first_shot_scale} type="number" onChange={(v) => setWeaponConfig(weaponId, 'first_shot_scale', v)} />
        <StatField label="Exponential Factor" value={config.exponential_factor} type="number" onChange={(v) => setWeaponConfig(weaponId, 'exponential_factor', v)} />
        <StatField label="DX" value={config.dx} type="number" onChange={(v) => setWeaponConfig(weaponId, 'dx', v)} />
        <StatField label="DY" value={config.dy} type="number" onChange={(v) => setWeaponConfig(weaponId, 'dy', v)} />
        <StatField label="Enable Recoil Control" value={config.enabled} type="checkbox" onChange={(v) => setWeaponConfig(weaponId, 'enabled', v)} />
    {/if}
</div>

<style>
    .stats-group {
        margin-top: 0.5rem;
        padding-left: 1rem;
        font-size: 0.9em;
    }
</style>
