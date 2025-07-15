<script lang="ts">
    import { PRESET_COLORS, isValidHexColor, normalizeHexColor } from '../lib/theme';
    
    export let value: string = '#bf0f70';
    export let label: string = 'Accent Color';
    export let onchange: (color: string) => void = () => {};
    
    let customColor = value;
    let showCustomInput = false;
    let isCustomValid = true;
    
    // Handle preset color selection
    const handlePresetSelect = (color: string) => {
        customColor = color;
        showCustomInput = false;
        onchange(color);
    };
    
    // Handle custom color input
    const handleCustomInput = (event: Event) => {
        const target = event.target as HTMLInputElement;
        const inputValue = target.value;
        customColor = inputValue;
        
        // Validate the color
        const normalizedColor = normalizeHexColor(inputValue);
        isCustomValid = isValidHexColor(normalizedColor);
        
        if (isCustomValid) {
            onchange(normalizedColor);
        }
    };
    
    // Handle custom color input via native color picker
    const handleColorPickerChange = (event: Event) => {
        const target = event.target as HTMLInputElement;
        customColor = target.value;
        isCustomValid = true;
        onchange(target.value);
    };
    
    const toggleCustomInput = () => {
        showCustomInput = !showCustomInput;
        if (showCustomInput) {
            customColor = value;
        }
    };
</script>

<div class="color-picker">
    <h3 class="color-picker-label">{label}</h3>
    
    <!-- Preset Colors Grid -->
    <div class="preset-colors">
        {#each PRESET_COLORS as preset}
            <button
                class="color-preset"
                class:selected={value === preset.value}
                style="background-color: {preset.value}"
                on:click={() => handlePresetSelect(preset.value)}
                title={preset.name}
                aria-label="Select {preset.name} color"
            >
                {#if value === preset.value}
                    <span class="checkmark">✓</span>
                {/if}
            </button>
        {/each}
        
        <!-- Custom Color Button -->
        <button
            class="color-preset custom-trigger"
            class:selected={showCustomInput}
            on:click={toggleCustomInput}
            title="Custom Color"
            aria-label="Enter custom color"
        >
            <span class="custom-icon">+</span>
        </button>
    </div>
    
    <!-- Custom Color Input -->
    {#if showCustomInput}
        <div class="custom-input-section">
            <div class="input-group">
                <input
                    type="text"
                    class="custom-color-input"
                    class:invalid={!isCustomValid}
                    bind:value={customColor}
                    on:input={handleCustomInput}
                    placeholder="#bf0f70"
                    aria-label="Custom hex color"
                />
                <input
                    type="color"
                    class="color-picker-native"
                    bind:value={customColor}
                    on:input={handleColorPickerChange}
                    aria-label="Color picker"
                />
            </div>
            {#if !isCustomValid}
                <p class="error-text">Please enter a valid hex color (e.g., #bf0f70)</p>
            {/if}
        </div>
    {/if}
    
    <!-- Current Color Preview -->
    <div class="current-color-preview">
        <div 
            class="color-preview" 
            style="background-color: {value}"
            title="Current accent color: {value}"
        ></div>
        <span class="color-value">{value}</span>
    </div>
</div>

<style>
.color-picker {
    margin: 16px;
}

.color-picker-label {
    color: var(--accent);
    margin-bottom: 20px;
}

.preset-colors {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(40px, 1fr));
    gap: 0.5rem;
    margin-bottom: 1rem;
    max-width: 300px;
}

.color-preset {
    width: 40px;
    height: 40px;
    border-radius: 8px;
    border: 2px solid transparent;
    cursor: pointer;
    transition: all 0.2s ease;
    display: flex;
    align-items: center;
    justify-content: center;
    position: relative;
}

.color-preset:hover {
    transform: scale(1.1);
    border-color: var(--fg);
}

.color-preset.selected {
    border-color: var(--fg);
    transform: scale(1.05);
}

.custom-trigger {
    background: linear-gradient(45deg, #f0f0f0 25%, transparent 25%), 
                linear-gradient(-45deg, #f0f0f0 25%, transparent 25%), 
                linear-gradient(45deg, transparent 75%, #f0f0f0 75%), 
                linear-gradient(-45deg, transparent 75%, #f0f0f0 75%);
    background-size: 8px 8px;
    background-position: 0 0, 0 4px, 4px -4px, -4px 0px;
}

@media (prefers-color-scheme: dark) {
    .custom-trigger {
        background: linear-gradient(45deg, #333 25%, transparent 25%), 
                    linear-gradient(-45deg, #333 25%, transparent 25%), 
                    linear-gradient(45deg, transparent 75%, #333 75%), 
                    linear-gradient(-45deg, transparent 75%, #333 75%);
        background-size: 8px 8px;
        background-position: 0 0, 0 4px, 4px -4px, -4px 0px;
    }
}

.custom-icon {
    font-size: 1.5rem;
    font-weight: bold;
    color: var(--fg);
}

.checkmark {
    color: white;
    font-weight: bold;
    font-size: 1.2rem;
    text-shadow: 1px 1px 2px rgba(0, 0, 0, 0.5);
}

.custom-input-section {
    background-color: rgba(255, 255, 255, 0.05);
    border-radius: 8px;
    padding: 1rem;
    margin-bottom: 1rem;
    border: 1px solid rgba(255, 255, 255, 0.1);
}

.input-group {
    display: flex;
    gap: 0.5rem;
    align-items: center;
}

.custom-color-input {
    flex: 1;
    padding: 0.5rem;
    border-radius: 6px;
    border: 1px solid rgba(255, 255, 255, 0.2);
    background-color: var(--card-bg);
    color: var(--fg);
    font-family: monospace;
    font-size: 0.9rem;
}

.custom-color-input.invalid {
    border-color: #dc3545;
    background-color: rgba(220, 53, 69, 0.1);
}

.custom-color-input:focus {
    outline: none;
    border-color: var(--accent);
    box-shadow: 0 0 0 2px rgba(191, 15, 112, 0.2);
}

.color-picker-native {
    width: 40px;
    height: 40px;
    border-radius: 6px;
    border: 1px solid rgba(255, 255, 255, 0.2);
    background: none;
    cursor: pointer;
}

.error-text {
    color: #dc3545;
    font-size: 0.8rem;
    margin: 0.5rem 0 0 0;
}

.current-color-preview {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.75rem;
    background-color: rgba(255, 255, 255, 0.03);
    border-radius: 8px;
    border: 1px solid rgba(255, 255, 255, 0.1);
}

.color-preview {
    width: 24px;
    height: 24px;
    border-radius: 4px;
    border: 1px solid rgba(255, 255, 255, 0.2);
}

.color-value {
    font-family: monospace;
    font-size: 0.9rem;
    color: var(--fg);
    opacity: 0.8;
}
</style>
