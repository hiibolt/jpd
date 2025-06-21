<script lang="ts">
    import { type Category, current_category_index } from "../stores/state";
    import { changeLoadout, changeCategory } from "../lib/api";

    import LoadoutCard from "./LoadoutCard.svelte";

    export let category: Category;
    export let index: number;

    function onclick() {
        changeCategory(index);
    }
    $: open = $current_category_index === index;
</script>

<button class="loadout-card" onclick={onclick}>
    <div class="category-name">
        {category.name}
    </div>
    {#if open}
        <div class="category-loadouts">
            {#if category.loadouts.length > 1}
                {#each category.loadouts as loadout, index}
                    <LoadoutCard name={loadout.name} onClick={() => changeLoadout(index)} />
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
