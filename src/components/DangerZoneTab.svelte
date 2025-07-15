<script lang="ts">
    import { resetConfigFromServer } from '../lib/api';

    // Component state
    let isResetting = false;
    let resetConfirmVisible = false;

    // Event handlers
    const handleResetConfig = async () => {
        if (!resetConfirmVisible) {
            resetConfirmVisible = true;
            return;
        }

        try {
            isResetting = true;
            resetConfirmVisible = false;
            await resetConfigFromServer();
        } catch (error) {
            console.error('Failed to reset config:', error);
        } finally {
            isResetting = false;
        }
    };

    const handleCancelReset = () => {
        resetConfirmVisible = false;
    };
</script>

<div class="tab-content danger-zone-tab">
    <h3>⚠️ Danger Zone</h3>
    <p class="danger-warning">
        The actions in this section can significantly modify your application data. 
        Please proceed with caution and make sure you understand the consequences.
    </p>

    <!-- Reset Configuration Section -->
    <div class="danger-section">
        <h4>Reset Game Data</h4>
        <p class="section-description">
            Reload fresh game configurations from the server. This will refresh all game data, loadouts, and weapon configurations while preserving your personal settings like keybinds and sensitivity.
        </p>
        
        {#if resetConfirmVisible}
            <div class="reset-confirm">
                <p class="warning-text">
                    ⚠️ This will reload all game data from the server. Your keybinds and mouse settings will be preserved.
                </p>
                <div class="reset-buttons">
                    <button 
                        class="btn btn-danger" 
                        on:click={handleResetConfig} 
                        disabled={isResetting}
                    >
                        {isResetting ? 'Reloading...' : 'Yes, Reload Game Data'}
                    </button>
                    <button 
                        class="btn btn-secondary" 
                        on:click={handleCancelReset} 
                        disabled={isResetting}
                    >
                        Cancel
                    </button>
                </div>
            </div>
        {:else}
            <button 
                class="btn btn-danger" 
                on:click={handleResetConfig} 
                disabled={isResetting}
            >
                {isResetting ? 'Reloading Game Data...' : 'Reload Game Data'}
            </button>
        {/if}
    </div>

    <!-- Future danger zone actions can be added here -->
    <!-- 
    <div class="danger-section">
        <h4>Clear All Data</h4>
        <p class="section-description">
            Permanently delete all application data including settings, keybinds, and configurations.
        </p>
        <button class="btn btn-danger" disabled>
            Clear All Data (Coming Soon)
        </button>
    </div>
    -->
</div>

<style>
    .danger-zone-tab {
        padding: 1rem;
    }

    .danger-warning {
        background-color: rgba(220, 53, 69, 0.1);
        border: 1px solid rgba(220, 53, 69, 0.3);
        border-radius: 8px;
        padding: 1rem;
        margin-bottom: 1.5rem;
        color: #dc3545;
        font-weight: 500;
        font-size: 0.875rem;
        line-height: 1.4;
    }

    .danger-section {
        margin-bottom: 2rem;
        padding: 1rem;
        border: 1px solid rgba(220, 53, 69, 0.2);
        border-radius: 8px;
        background-color: rgba(220, 53, 69, 0.05);
    }

    .danger-section h4 {
        margin: 0 0 0.5rem 0;
        color: #dc3545;
        font-size: 1.1rem;
    }

    .section-description {
        font-size: 0.875rem;
        color: var(--fg);
        opacity: 0.8;
        margin-bottom: 1rem;
        line-height: 1.4;
    }

    .reset-confirm {
        background-color: rgba(220, 53, 69, 0.15);
        border: 1px solid rgba(220, 53, 69, 0.4);
        border-radius: 8px;
        padding: 1rem;
        margin-bottom: 1rem;
    }

    .warning-text {
        color: #dc3545;
        font-weight: 600;
        margin-bottom: 1rem;
        font-size: 0.875rem;
    }

    .reset-buttons {
        display: flex;
        gap: 0.75rem;
        flex-wrap: wrap;
    }
</style>
