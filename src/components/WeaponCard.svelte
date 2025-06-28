<script lang="ts">
    import type { Weapon } from '../stores/state';
    import WeaponStats from './WeaponStats.svelte';

    export let weaponId: string;
    export let active: boolean;
    export let shooting: boolean;
    export let weapon: Weapon;

    let open = false;

    function toggle() {
        open = !open;
    }
</script>

<div class="weapon-card {active ? 'active' : ''} {open ? 'open' : ''}">
    <div class="weapon-header">
        <span><b>{weapon.config.name}</b></span>
        <span class="weapon-status {shooting ? 'shooting' : 'not-shooting'}"></span>
    </div>

    <button class="weapon-card-toggle" onclick={toggle}>
        {open ? 'Hide Details' : 'Show Details'}
    </button>

    {#if weapon}
        {#if open}
            <WeaponStats weaponId={weaponId} config={weapon.config} type={weapon.type} />
        {/if}
    {:else}
        <p>Weapon data unavailable</p>
    {/if}
</div>

<style>
    .weapon-card {
        background: var(--card-bg);
        border: 1px solid #ccc;
        padding: 0.75rem;
        border-radius: 6px;
        transition: transform 0.2s, border 0.2s;
    }
    .weapon-card.active {
        transform: scale(1.05);
        border-color: var(--accent);
    }
    .weapon-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
    }
    .weapon-status {
        width: 16px;
        height: 16px;
        border-radius: 50%;
        animation: pulse 1s infinite alternate ease-in-out;
    }
    .weapon-status.shooting {
        background: var(--shooting);
    }
    .weapon-status.not-shooting {
        background: var(--not-shooting);
        animation: none;
    }
    .weapon-card-toggle {
        background: none;
        border: none;
        color: var(--accent);
        cursor: pointer;
        font-size: 0.85rem;
        align-self: flex-end;
        margin-top: 0.25rem;
    }
    @keyframes pulse {
        from { transform: scale(1); }
        to { transform: scale(1.3); }
    }
</style>
