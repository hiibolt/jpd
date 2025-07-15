// Theme manager for handling dynamic color changes
import { config } from '../stores/state';
import { updateAccentColor } from './theme';

/**
 * Initialize theme system and set up reactive color updates
 */
export function initializeTheme(): void {
    // Load saved accent color from localStorage (with error handling)
    try {
        if (typeof localStorage !== 'undefined') {
            const savedAccentColor = localStorage.getItem('accent_color');
            if (savedAccentColor) {
                // Update the config store with the saved color
                config.update(currentConfig => ({
                    ...currentConfig,
                    theme_config: {
                        ...currentConfig.theme_config,
                        accent_color: savedAccentColor
                    }
                }));
            }
        }
    } catch (error) {
        console.warn('Failed to load accent color from localStorage:', error);
    }

    // Subscribe to config changes and update CSS custom properties
    config.subscribe(($config) => {
        if ($config.theme_config?.accent_color) {
            updateAccentColor($config.theme_config.accent_color);
        }
    });
}

/**
 * Apply theme immediately for initial load
 */
export function applyInitialTheme(accentColor: string): void {
    updateAccentColor(accentColor);
}
