<script lang="ts">
    import type { Game } from "../stores/state";
    import CategoryCard from "./CategoryCard.svelte";

    export let game: Game;
    export let onClick: () => void;
</script>

<button class="loadout-card" onclick={onClick}>
    <div class="game-name">
        {game.name}
    </div>
    <div class="game-categories">
        {#if game.categories.length > 0}
            {#each game.categories as category}
                <CategoryCard name={category.name} onClick={() => onClick()} />
            {/each}
        {:else}
            <span class="no-categories">No Categories</span>
        {/if}
    </div>
</button>

<style>
    .game-name {
        font-weight: bold;
        font-size: 1.2rem;
    }
    .game-categories {
        display: flex;
        flex-wrap: wrap;
        gap: 0.5rem;
        margin-top: 0.5rem;
    }
    .loadout-card {
        color: var(--fg);
        width: 100%;
        padding: 0.5rem 0.75rem;
        white-space: nowrap;
        overflow: hidden;
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
