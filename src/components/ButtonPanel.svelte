<script>
    import { version } from "../stores/state";

    export let currentPage = 'home';
</script>

<div class="account-panel card">
    <div class="username">
        Product of <b>CLC Services</b>
    </div>
    <div class="button-group">
        {#if currentPage !== 'home'}
            <a href="/"><button data-tooltip="Home">🏠</button></a>
        {/if}
        {#if currentPage !== 'settings'}
            <a href="/settings"><button data-tooltip="Settings">⚙️</button></a>
        {/if}
        <button data-tooltip="Version"><b>{$version}</b></button>
    </div>
</div>

<style>
    .account-panel {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 1rem;
        padding: 0.5rem 1rem;
        flex-shrink: 0;
    }

    .username {
        font-size: 0.85em;
        opacity: 0.75;
    }

    .button-group {
        display: flex;
        gap: 0.5rem;
    }

    .button-group button {
        background: var(--accent);
        color: white;
        border: none;
        border-radius: 4px;
        padding: 0.5rem 1rem;
        cursor: pointer;
        position: relative;
    }

    .button-group button::after {
        content: attr(data-tooltip);
        position: absolute;
        bottom: 125%;
        left: 50%;
        transform: translateX(-50%);
        background: #333;
        color: white;
        padding: 0.25rem 0.5rem;
        border-radius: 4px;
        font-size: 0.75rem;
        white-space: nowrap;
        opacity: 0;
        pointer-events: none;
        transition: opacity 0.2s;
    }

    .button-group button:hover::after {
        opacity: 1;
    }
</style>
