<script lang="ts">
    import { type Category, current_category_index, current_loadout_index } from "../../stores/state";
    import { changeLoadout, changeCategory } from "../../lib/api";

    import LoadoutCard from "./LoadoutCard.svelte";

    export let category: Category;
    export let index: number;

    function onclick() {
        changeCategory(index);
    }
    $: open = $current_category_index === index;
</script>

<button class="category-card {open ? 'active' : ''}" onclick={onclick}>
    <div class="category-name">
        {category.name}
    </div>
    {#if open}
        <div class="category-loadouts">
            {#if category.loadouts.length > 1}
                {#each category.loadouts as loadout, index}
                    <LoadoutCard 
                        loadout={loadout} 
                        onClick={() => changeLoadout(index)} 
                        active={$current_loadout_index === index}
                    />
                {/each}
            {:else}
                <p>Loading loadouts...</p>
            {/if}
        </div>
    {/if}
</button>

<style>
    .category-name {
        font-weight: bold;
        font-size: 1.2rem;
    }
    .category-loadouts {
        display: flex;
        flex-wrap: wrap;
        gap: 0.15rem;
        margin-top: 0.5rem;
    }
    .category-card {
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
    .category-card.active {
        transform: scale(1.01);
        border-color: var(--accent);
    }
    .category-card:hover {
        transform: translateY(-2px);
        box-shadow: 0 4px 15px rgba(0, 0, 0, 0.2);
    }
</style>
