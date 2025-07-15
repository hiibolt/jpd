// Utility functions for theme management

/**
 * Updates CSS custom properties for theme colors
 */
export function updateAccentColor(color: string): void {
    const root = document.documentElement;
    root.style.setProperty('--accent', color);
    
    // Also update any derived colors if needed
    // For example, a slightly transparent version for hover states
    const rgb = hexToRgb(color);
    if (rgb) {
        root.style.setProperty('--accent-rgb', `${rgb.r}, ${rgb.g}, ${rgb.b}`);
        root.style.setProperty('--accent-transparent', `rgba(${rgb.r}, ${rgb.g}, ${rgb.b}, 0.1)`);
    }
}

/**
 * Converts hex color to RGB values
 */
function hexToRgb(hex: string): { r: number; g: number; b: number } | null {
    const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
    return result ? {
        r: parseInt(result[1], 16),
        g: parseInt(result[2], 16),
        b: parseInt(result[3], 16)
    } : null;
}

/**
 * Validates if a string is a valid hex color
 */
export function isValidHexColor(color: string): boolean {
    return /^#?([A-Fa-f0-9]{6}|[A-Fa-f0-9]{3})$/.test(color);
}

/**
 * Ensures hex color has # prefix
 */
export function normalizeHexColor(color: string): string {
    return color.startsWith('#') ? color : `#${color}`;
}

/**
 * Predefined color options for quick selection
 */
export const PRESET_COLORS = [
    { name: 'JPD Pink', value: '#bf0f70' },
    { name: 'Electric Blue', value: '#007acc' },
    { name: 'Emerald Green', value: '#10b981' },
    { name: 'Sunset Orange', value: '#f59e0b' },
    { name: 'Purple Heart', value: '#8b5cf6' },
    { name: 'Crimson Red', value: '#dc2626' },
    { name: 'Teal', value: '#0d9488' },
    { name: 'Rose Gold', value: '#e879f9' },
    { name: 'Cyber Yellow', value: '#eab308' },
    { name: 'Ocean Blue', value: '#0ea5e9' },
    { name: 'Forest Green', value: '#059669' },
    { name: 'Lavender', value: '#a855f7' }
] as const;
