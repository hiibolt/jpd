<script lang="ts">
    import ColorPicker from '../ColorPicker.svelte';
    import { config } from '../../stores/state';
    import { changeAccentColor } from '../../lib/api';

    const handleAccentColorChange = (color: string) => {
        changeAccentColor(color);
    };
</script>

<div class="visuals-tab tab-content" role="tabpanel" id="tabpanel-visuals" aria-labelledby="tab-visuals">
    <!-- Theme Configuration -->
    <div class="theme-config-section">
        <h2>🎨 Theming and Visual Settings</h2>
        <p class="section-description">
            Customize the visual appearance of the application to match your preferences.
        </p>
        
        <div class="card">
            <ColorPicker
                value={$config.theme_config?.accent_color || '#bf0f70'}
                label="Accent Color"
                onchange={handleAccentColorChange}
            />
        </div>
    </div>
    
    <!-- Future visual settings can be added here -->
    <div class="future-settings">
        <h3>🔮 More Visual Options Coming Soon</h3>
        <p class="coming-soon-text">
            We're working on additional customization options including:
        </p>
        <ul class="features-list">
            <li>🌙 Dark/Light theme toggle</li>
            <li>🎭 Custom background options</li>
            <li>📏 UI scaling preferences</li>
            <li>⚡ Animation intensity settings</li>
        </ul>
    </div>
</div>

<style>
.visuals-tab {
    padding: 0.5rem 0;
}

.theme-config-section {
    text-align: center;
    align-items: center;
    margin-bottom: 2rem;
}

.theme-config-section h2 {
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

.future-settings {
    padding: 1.5rem;
    background-color: rgba(255, 255, 255, 0.03);
    border-radius: 12px;
    border: 1px solid rgba(255, 255, 255, 0.08);
    margin-top: 2rem;
}

.future-settings h3 {
    color: var(--accent);
    margin-bottom: 0.75rem;
    font-size: 1rem;
    opacity: 0.9;
}

.coming-soon-text {
    font-size: 0.875rem;
    color: var(--fg);
    opacity: 0.7;
    margin-bottom: 1rem;
    line-height: 1.4;
}

.features-list {
    list-style: none;
    padding: 0;
    margin: 0;
}

.features-list li {
    padding: 0.5rem 0;
    font-size: 0.875rem;
    color: var(--fg);
    opacity: 0.6;
    border-bottom: 1px solid rgba(255, 255, 255, 0.05);
    transition: opacity 0.2s ease;
}

.features-list li:last-child {
    border-bottom: none;
}

.features-list li:hover {
    opacity: 0.8;
}

@media (prefers-color-scheme: dark) {
    .future-settings {
        background-color: rgba(255, 255, 255, 0.02);
        border-color: rgba(255, 255, 255, 0.05);
    }
}
</style>
