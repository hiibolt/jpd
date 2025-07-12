<script lang="ts">
    import Titlebar from '../components/Titlebar.svelte';
    import Banner from '../components/Banner.svelte';
    import ButtonPanel from '../components/ButtonPanel.svelte';
    import Background from '../components/Background.svelte';
    import ConfigGroup from '../components/ConfigGroup.svelte';
    import { games, config } from '../stores/state';
    import { resetConfigFromServer } from '../lib/api';

    let keybindConfigOptions: any[] = [
        { label: 'Primary Weapon', description: 'Switches to primary weapon', type: 'char', key: 'primary_weapon', value: $config.keybinds.primary_weapon },
        { label: 'Secondary Weapon', description: 'Switches to secondary weapon', type: 'char', key: 'secondary_weapon', value: $config.keybinds.secondary_weapon },
        { label: 'Alternative Fire', description: 'Bind your shoot key to this in-game for autofire to work', type: 'char', key: 'alternative_fire', value: $config.keybinds.alternative_fire }
    ];

    let isResetting = false;
    let resetConfirmVisible = false;

    async function handleResetConfig() {
        if (!resetConfirmVisible) {
            resetConfirmVisible = true;
            return;
        }

        isResetting = true;
        resetConfirmVisible = false;
        
        await resetConfigFromServer();

		isResetting = false;
    }

    function cancelReset() {
        resetConfirmVisible = false;
    }
</script>

<Background />
<main class="container">
    <Titlebar />
    <Banner />

    <div class="main-layout">
        <!-- Loadouts -->
        <div class="left-column card">
            <h2>Configuration Options</h2>
            <ConfigGroup configOptions={keybindConfigOptions} label="Keybinds" />
            
            <div class="reset-config-section">
                <h3>Reset Game Data</h3>
                <p class="reset-description">
                    Reload fresh game configurations from the server. This will refresh all game data, loadouts, and weapon configurations while preserving your personal settings like keybinds and sensitivity.
                </p>
                
                {#if resetConfirmVisible}
                    <div class="reset-confirm">
                        <p class="warning-text">⚠️ This will reload all game data from the server. Your keybinds and mouse settings will be preserved.</p>
                        <div class="reset-buttons">
                            <button class="reset-btn danger" on:click={handleResetConfig} disabled={isResetting}>
                                {isResetting ? 'Reloading...' : 'Yes, Reload Game Data'}
                            </button>
                            <button class="reset-btn cancel" on:click={cancelReset} disabled={isResetting}>Cancel</button>
                        </div>
                    </div>
                {:else}
                    <button class="reset-btn primary" on:click={handleResetConfig} disabled={isResetting}>
                        {isResetting ? 'Reloading Game Data...' : 'Reload Game Data'}
                    </button>
                {/if}
            </div>
        </div>

        <!-- Active Loadout -->
        <div class="right-column">
            <div class="card upper-right-card">
                <h3>Settings</h3>
                <p>
                    Manage your settings and preferences.
                    <br>
                    <br>
                    Need help? Reach out to CLC and open a support ticket - we're here to help you play best.
                </p>
            </div>

            <ButtonPanel currentPage="settings"/>
        </div>
    </div>
</main>

<style>
:root {
  --fg: #222;
  --card-bg: rgba(255, 255, 255, 0.8);
  --card-shadow: 0 2px 10px rgba(0, 0, 0, 0.15);
  --accent: #0077ff;
  --shooting: #e53935;
  --not-shooting: #333;
  backdrop-filter: blur(20px);
}

@media (prefers-color-scheme: dark) {
  :root {
    --fg: #f6f6f6;
    --card-bg: rgba(30, 30, 30, 0.8);
    --card-shadow: 0 2px 10px rgba(0, 0, 0, 0.5);
  }
}

main.container {
  color: var(--fg);
  font-family: Inter, sans-serif;
  height: 100vh;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background: transparent; /* No background - acrylic look */
}

.main-layout {
  display: grid;
  grid-template-columns: 2fr 1fr;
  gap: 1rem;
  padding: 1rem;
  flex: 1;
  height: calc(100vh - 80px); /* titlebar + banner space */
  box-sizing: border-box;
  overflow: hidden;
}

.left-column {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(120px, 1fr));
  gap: 0.75rem;
  overflow-y: auto;
  padding: 0.5rem;
  text-align: center;
}

.right-column {
  display: flex;
  flex-direction: column;
  height: 100%;
  gap: 1rem;
}

.upper-right-card {
  flex: 9;
  display: flex;
  flex-direction: column;
  overflow-y: auto;
  text-align: center;
}
.upper-right-card p {
  margin: 0.5rem 0;
  line-height: 1.5;
  font-weight: 500;
}

.card {
  background-color: var(--card-bg);
  border-radius: 10px;
  box-shadow: var(--card-shadow);
  padding: 1rem;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

h3 {
  margin-top: 0;
}

.reset-config-section {
  margin-top: 2rem;
  padding-top: 1.5rem;
  border-top: 1px solid rgba(128, 128, 128, 0.3);
}

.reset-description {
  font-size: 0.875rem;
  color: var(--fg);
  opacity: 0.8;
  margin-bottom: 1rem;
  line-height: 1.4;
}

.reset-confirm {
  background-color: rgba(220, 53, 69, 0.1);
  border: 1px solid rgba(220, 53, 69, 0.3);
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

.reset-btn {
  padding: 0.5rem 1rem;
  border: none;
  border-radius: 6px;
  font-size: 0.875rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
  min-width: 120px;
}

.reset-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.reset-btn.primary {
  background-color: var(--accent);
  color: white;
}

.reset-btn.primary:hover:not(:disabled) {
  background-color: #0056cc;
}

.reset-btn.danger {
  background-color: #dc3545;
  color: white;
}

.reset-btn.danger:hover:not(:disabled) {
  background-color: #c82333;
}

.reset-btn.cancel {
  background-color: #6c757d;
  color: white;
}

.reset-btn.cancel:hover:not(:disabled) {
  background-color: #5a6268;
}
</style>

