<script lang="ts">
    import CategoryCard from "./CategoryCard.svelte";

    import { type Game, current_game_index } from "../stores/state";
    import { changeGame } from "$lib/api";

    export let game: Game;
    export let index: number;

    function onclick() {
        changeGame(index);
    }
    $: open = $current_game_index === index;
</script>

<button class="game-card" onclick={onclick}>
    <div class="game-name">
        {game.name}
    </div>
    {#if open}
        <div class="game-categories">
            {#if game.categories.length > 0}
                {#each game.categories as category, index}
                    <CategoryCard
                        category={category}
                        index={index}
                    />
                {/each}
            {:else}
                <span class="no-categories">No Categories</span>
            {/if}
        </div>
    {/if}
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
    .game-card {
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
    .game-card:hover {
        transform: translateY(-2px);
        box-shadow: 0 4px 15px rgba(0, 0, 0, 0.2);
    }
</style>
