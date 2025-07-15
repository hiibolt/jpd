<script lang="ts">
    import Titlebar from '../components/Titlebar.svelte';
    import ButtonPanel from '../components/ButtonPanel.svelte';
    import Background from '../components/Background.svelte';
    import ConfigGroup from '../components/ConfigGroup.svelte';
    import StatField from '../components/StatField.svelte';
    import { config } from '../stores/state';
    import type { KeybindConfigOption } from '../lib/types';
    import { 
        resetConfigFromServer, 
        changeHorizontalMultiplier, 
        changeVerticalMultiplier, 
        changeAcogHorizontalMultiplier, 
        changeAcogVerticalMultiplier, 
        changeScrollWheelWeaponSwap 
    } from '../lib/api';
	import { open } from '@tauri-apps/plugin-shell';

    // Reactive configuration options
    let keybindConfigOptions: KeybindConfigOption[];
    $: keybindConfigOptions = [
        { 
            label: 'Primary Weapon', 
            description: 'Switches to primary weapon', 
            type: 'char', 
            key: 'primary_weapon', 
            value: $config.keybinds.primary_weapon 
        },
        { 
            label: 'Secondary Weapon', 
            description: 'Switches to secondary weapon', 
            type: 'char', 
            key: 'secondary_weapon', 
            value: $config.keybinds.secondary_weapon 
        },
        { 
            label: 'Alternative Fire', 
            description: 'Bind your shoot key to this in-game for autofire to work', 
            type: 'char', 
            key: 'alternative_fire', 
            value: $config.keybinds.alternative_fire 
        }
    ];

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

    const handleScrollWheelToggle = (event: Event) => {
        const target = event.target as HTMLInputElement;
        changeScrollWheelWeaponSwap(target.checked);
    };

    const handleOpenDiscord = () => {
        open("https://discord.gg/pulldown");
    };
</script>

<Background />
<main class="container">
    <Titlebar />
	<br>

    <div class="main-layout">
        <!-- Loadouts -->
        <div class="left-column card scrollable">
            <h2>Configuration Options</h2>
            <ConfigGroup configOptions={keybindConfigOptions} label="Keybinds" />
            
            <!-- Mouse Sensitivity Settings -->
            <div class="mouse-config-section">
                <h3>Mouse Sensitivity Multipliers</h3>
                <div class="sensitivity-fields">
                    <StatField
                        label="1x Vertical Sensitivity Multiplier"
                        value={$config.mouse_config.vertical_multiplier}
                        type="number"
                        onChange={(v) => changeVerticalMultiplier(v)}
                    />
                    <StatField
                        label="1x Horizontal Sensitivity Multiplier"
                        value={$config.mouse_config.horizontal_multiplier}
                        type="number"
                        onChange={(v) => changeHorizontalMultiplier(v)}
                    />
                    <StatField
                        label="2.5x Vertical Sensitivity Multiplier"
                        value={$config.mouse_config.acog_vertical_multiplier}
                        type="number"
                        onChange={(v) => changeAcogVerticalMultiplier(v)}
                    />
                    <StatField
                        label="2.5x Horizontal Sensitivity Multiplier"
                        value={$config.mouse_config.acog_horizontal_multiplier}
                        type="number"
                        onChange={(v) => changeAcogHorizontalMultiplier(v)}
                    />
                </div>
                
                <div class="checkbox-field">
                    <label class="checkbox-label">
                        <input 
                            type="checkbox" 
                            bind:checked={$config.mouse_config.scroll_wheel_weapon_swap}
                            on:change={handleScrollWheelToggle}
                        />
                        <span class="checkbox-text">Enable Scroll Wheel Weapon Swap</span>
                    </label>
                    <p class="checkbox-description">
                        Allow mouse wheel to cycle between primary and secondary weapons
                    </p>
                </div>
            </div>
            
            <div class="reset-config-section">
                <h3>Reset Game Data</h3>
                <p class="reset-description">
                    Reload fresh game configurations from the server. This will refresh all game data, loadouts, and weapon configurations while preserving your personal settings like keybinds and sensitivity.
                </p>
                
                {#if resetConfirmVisible}
                    <div class="reset-confirm">
                        <p class="warning-text">
                            ⚠️ This will reload all game data from the server. Your keybinds and mouse settings will be preserved.
                        </p>
                        <div class="reset-buttons">
                            <button 
                                class="reset-btn danger" 
                                on:click={handleResetConfig} 
                                disabled={isResetting}
                            >
                                {isResetting ? 'Reloading...' : 'Yes, Reload Game Data'}
                            </button>
                            <button 
                                class="reset-btn cancel" 
                                on:click={handleCancelReset} 
                                disabled={isResetting}
                            >
                                Cancel
                            </button>
                        </div>
                    </div>
                {:else}
                    <button 
                        class="reset-btn primary" 
                        on:click={handleResetConfig} 
                        disabled={isResetting}
                    >
                        {isResetting ? 'Reloading Game Data...' : 'Reload Game Data'}
                    </button>
                {/if}
            </div>
        </div>

        <!-- Active Loadout -->
        <div class="right-column">	
            <div class="card upper-right-card scrollable">
                <h3>Settings</h3>
                <p>
                    Manage your settings and preferences.
                    <br>
                    <br>
                    Need help? Reach out to JPD staff in the 
                    <button class="link-button" on:click={handleOpenDiscord}>
                        Discord
                    </button> 
                    and open a support ticket - we're here to help you play best.
                </p>
				<div class="username">
					Maintained by <b>@hiibolt</b> with &lt;3
				</div>
            </div>

            <ButtonPanel currentPage="settings"/>
        </div>
    </div>
</main>

<style>
.left-column {
  text-align: center;
}

.upper-right-card {
  text-align: center;
}

.upper-right-card p {
  margin: 0.5rem 0;
  line-height: 1.5;
  font-weight: 500;
}

.username {
  font-size: 0.85em;
  opacity: 0.75;
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
  background-color: var(--accent);
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

.mouse-config-section {
  margin: 2rem 0;
}

.mouse-config-section h3 {
  color: var(--accent);
  margin-bottom: 1rem;
}

.sensitivity-fields {
  display: flex;
  flex-direction: column;
}

.checkbox-field {
  margin-top: 1.5rem;
  padding: 1rem;
  background-color: rgba(255, 255, 255, 0.05);
  border-radius: 8px;
  border: 1px solid rgba(255, 255, 255, 0.1);
}

.checkbox-label {
  display: flex;
  align-items: center;
  cursor: pointer;
  gap: 0.75rem;
}

.checkbox-label input[type="checkbox"] {
  width: 18px;
  height: 18px;
  accent-color: var(--accent);
  cursor: pointer;
}

.checkbox-text {
  font-size: 1rem;
  font-weight: 500;
  color: var(--fg);
}

.checkbox-description {
  font-size: 0.875rem;
  color: var(--fg);
  opacity: 0.7;
  margin: 0.5rem 0 0 0;
  line-height: 1.4;
}
</style>

