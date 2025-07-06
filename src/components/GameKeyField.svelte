<script lang="ts">
    import { type Game } from "../stores/state";

    export let game: Game;
</script>

<div class="game-key hidden">
    <form>
        <input
            type="text"
            bind:value={game.key}
            placeholder="Enter game key"
        />
        <button
            type="button"
        >
            Change Game
        </button>
    </form>
</div>

<style>
    .hidden {
        opacity: 0.5;
        transition: opacity 0.5s ease;
    }
    .hidden:hover {
        opacity: 1;
    }

    .game-key {
        width: 100%;
        display: flex;
        justify-content: space-between;
    }
    .game-key input {
        width: 100%;
        background-color: var(--input-bg);
        border: 1px solid var(--input-border);
        color: var(--fg);
        padding: 0.25rem;
        border-radius: 4px;
    }
</style>