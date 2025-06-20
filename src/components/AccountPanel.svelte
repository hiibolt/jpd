<script>
  // Optional: you can pass the username in via props
</script>

<div class="account-panel card">
    <div class="username">
        <b>@hiibolt</b>
    </div>
    <div class="button-group">
        <button data-tooltip="Settings">⚙️</button>
        <button data-tooltip="Account">👤</button>
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
