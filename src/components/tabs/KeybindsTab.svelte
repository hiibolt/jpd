<script lang="ts">
    import ConfigGroup from '../ConfigGroup.svelte';
    import StatField from '../StatField.svelte';
    import { config } from '../../stores/state';
    import type { KeybindConfigOption } from '../../lib/types';
    import { 
        changeHorizontalMultiplier, 
        changeVerticalMultiplier, 
        changeAcogHorizontalMultiplier, 
        changeAcogVerticalMultiplier, 
        changeScrollWheelWeaponSwap 
    } from '../../lib/api';

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

    const handleScrollWheelToggle = (event: Event) => {
        const target = event.target as HTMLInputElement;
        changeScrollWheelWeaponSwap(target.checked);
    };
</script>

<div class="keybinds-tab tab-content" role="tabpanel" id="tabpanel-keybinds" aria-labelledby="tab-keybinds">
    <h2>⌨️ Mouse & Keyboard Keybind Settings</h2>
    <p class="section-description">
        Customize the keybinds for various actions in the application.
    </p>
    
    <div class="card">
        <ConfigGroup configOptions={keybindConfigOptions} label="Weapons Keybinds" />
    </div>
    
    <!-- Mouse Sensitivity Settings -->
    <div class="mouse-config-section card">
        <h4>Mouse Sensitivity Multipliers</h4>
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
</div>

<style>
.keybinds-tab {
    padding: 0.5rem 0;
}

.keybinds-tab h2 {
    color: white;
    margin-bottom: 0.5rem;
    font-size: 1.1rem;
    gap: 0.5rem;
}

.section-description {
    font-size: 0.9rem;
    color: var(--fg);
    opacity: 0.8;
    margin-bottom: 1.5rem;
    line-height: 1.5;
}

.mouse-config-section {
    margin: 2rem 0;
}

.mouse-config-section h4 {
    color: var(--accent);
    margin-bottom: 1rem;
    font-size: 1.1rem;
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
