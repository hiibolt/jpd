<script lang="ts">
    import { minimize, maximize, close } from '../lib/tauriHelper';
</script>

<div class="titlebar" data-tauri-drag-region>
    <button class="titlebar-button" aria-label="Minimize" on:click={minimize}>
        <img src="https://api.iconify.design/mdi:window-minimize.svg" alt="minimize" />
    </button>
    <button class="titlebar-button" aria-label="Maximize" on:click={maximize}>
        <img src="https://api.iconify.design/mdi:window-maximize.svg" alt="maximize" />
    </button>
    <button class="titlebar-button" aria-label="Close" on:click={close}>
        <img src="https://api.iconify.design/mdi:close.svg" alt="close" />
    </button>
</div>

<style>
    .titlebar {
        height: 30px;
        display: flex;
        justify-content: flex-end;
        align-items: center;
        position: fixed;
        top: 0;
        left: 0;
        right: 0;
        padding: 0 8px;
        background: transparent;
        z-index: 1000;
    }
    .titlebar-button {
        background: transparent;
        border: none;
        width: 20px;
        height: 20px;
        margin-left: 8px;
    }
</style>
