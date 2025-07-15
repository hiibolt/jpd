<script lang="ts">
    import { type Game, type KeyStatus } from "../stores/state";
    import { submitGameKey } from "$lib/api";

    export let game: Game;
    
    let showKeyInput = false;
    let keyInput = "";
    let isSubmitting = false;
    
    function getStatusDisplay(status: KeyStatus | undefined): { text: string; class: string } {
        if (!status) {
            if (game.key) {
                return { text: "Key status unknown", class: "status-unknown" };
            } else {
                return { text: "No key provided", class: "status-no-key" };
            }
        }
        
        switch (status.type) {
            case 'Valid':
                const expirationDate = new Date(status.timestamp * 1000);
                const now = new Date();
                if (expirationDate > now) {
                    return { 
                        text: `Valid until ${expirationDate.toLocaleDateString()}`, 
                        class: "status-valid" 
                    };
                } else {
                    return { text: "Expired", class: "status-expired" };
                }
            case 'Invalid':
                return { text: "Invalid key", class: "status-invalid" };
            case 'Expired':
                const expiredDate = new Date(status.timestamp * 1000);
                return { 
                    text: `Expired on ${expiredDate.toLocaleDateString()}`, 
                    class: "status-expired" 
                };
            case 'Banned':
                return { text: "Key is banned", class: "status-banned" };
            default:
                return { text: "Unknown status", class: "status-unknown" };
        }
    }
    
    async function handleSubmitKey() {
        if (!keyInput.trim()) return;
        
        isSubmitting = true;
        try {
            const result = await submitGameKey(game.name, keyInput.trim());
            if (result) {
                console.log(`Key submitted for ${game.name}, games updated`);
                // The games store is automatically updated by the submitGameKey function
            }
            keyInput = "";
            showKeyInput = false;
        } catch (error) {
            console.error('Failed to submit key:', error);
        } finally {
            isSubmitting = false;
        }
    }
    
    function toggleKeyInput() {
        showKeyInput = !showKeyInput;
        if (!showKeyInput) {
            keyInput = "";
        }
    }
    
    $: statusInfo = getStatusDisplay(game.key_status);
</script>

<div class="game-key-section">
    <div class="key-status">
        <span class="status-text {statusInfo.class}">
            {statusInfo.text}
        </span>
        <button 
            type="button" 
            class="toggle-key-input"
            on:click={toggleKeyInput}
            aria-label={showKeyInput ? "Hide key input" : "Show key input"}
        >
            {showKeyInput ? "Cancel" : "Change Key"}
        </button>
    </div>
    
    {#if showKeyInput}
        <div class="key-input-form">
            <input
                type="text"
                bind:value={keyInput}
                placeholder="Enter game key"
                disabled={isSubmitting}
                on:keydown={(e) => e.key === 'Enter' && handleSubmitKey()}
            />
            <button
                type="button"
                on:click={handleSubmitKey}
                disabled={isSubmitting || !keyInput.trim()}
                class="submit-button"
            >
                {isSubmitting ? "Submitting..." : "Submit"}
            </button>
        </div>
    {/if}
</div>

<style>
    .game-key-section {
        margin: 0.5rem;
    }
    
    .key-status {
        display: flex;
        justify-content: space-between;
        align-items: center;
        gap: 2.5rem;
        text-align: left;
        margin-left: 20px;
        margin-right: 20px;
    }
    
    .status-text {
        font-size: 0.9rem;
        font-weight: 500;
        flex: 1;
    }
    
    .status-valid {
        color: #d4338c;
    }
    
    .status-invalid,
    .status-banned {
        color: #f87171;
    }
    
    .status-expired {
        color: #fb923c;
    }
    
    .status-unknown,
    .status-no-key {
        color: #94a3b8;
    }
    
    .toggle-key-input {
        background-color: var(--input-bg);
        border: 1px solid var(--input-border);
        color: var(--fg);
        padding: 0.25rem 0.5rem;
        border-radius: 4px;
        font-size: 0.8rem;
        cursor: pointer;
        transition: background-color 0.2s;
        text-align: right;
    }
    
    .toggle-key-input:hover {
        background-color: var(--accent);
        border-color: var(--accent);
    }
    
    .key-input-form {
        display: flex;
        gap: 0.5rem;
        margin-top: 0.5rem;
        animation: slideDown 0.2s ease-out;
    }
    
    @keyframes slideDown {
        from {
            opacity: 0;
            transform: translateY(-10px);
        }
        to {
            opacity: 1;
            transform: translateY(0);
        }
    }
    
    .key-input-form input {
        flex: 1;
        background-color: var(--input-bg);
        border: 1px solid var(--input-border);
        color: var(--fg);
        padding: 0.375rem;
        border-radius: 4px;
        font-size: 0.9rem;
    }
    
    .key-input-form input:focus {
        outline: none;
        border-color: var(--accent);
        box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.3);
    }
    
    .submit-button {
        background-color: var(--accent);
        border: 1px solid var(--accent);
        color: white;
        padding: 0.375rem 0.75rem;
        border-radius: 4px;
        font-size: 0.9rem;
        cursor: pointer;
        font-weight: 500;
        transition: background-color 0.2s;
        white-space: nowrap;
    }
    
    .submit-button:hover:not(:disabled) {
        background-color: #3b82f6;
        border-color: #3b82f6;
    }
    
    .submit-button:disabled {
        opacity: 0.6;
        cursor: not-allowed;
    }
</style>