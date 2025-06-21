<script lang="ts">
    import type { Loadout } from "../stores/state";

    export let loadout: Loadout;
    export let onClick: () => void;
</script>

<button class="loadout-card" onclick={onClick}>
    {#if loadout.icon_url }
        <img src={loadout.icon_url} alt={loadout.name} width="40" />
    {/if}
    {#if !loadout.icon_only }
        {loadout.name}
    {/if}
</button>

<style>
    .loadout-card {
        color: var(--fg);
        border: none;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
        background: var(--card-bg);
        cursor: pointer;
        text-align: center;
        transition: transform 0.2s, box-shadow 0.2s;
    }
    .loadout-card:hover {
        transform: translateY(-2px);
        box-shadow: 0 4px 15px rgba(0, 0, 0, 0.2);
    }
</style>
