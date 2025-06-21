<script lang="ts">
    export let name: string;
    export let onClick: () => void;
</script>

<button class="loadout-card" onclick={onClick}>
    {name}
</button>

<style>
    .loadout-card {
        color: var(--fg);
        width: 100%;
        padding: 0.5rem 0.75rem;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
        border-radius: 8px;
        border: 1px solid #ccc;
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
