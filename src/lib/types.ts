// Common TypeScript types and interfaces for the application

export interface Game {
    name: string;
    categories: Category[];
    weapons: Record<string, Weapon>;
}

export interface Category {
    name: string;
    loadouts: Loadout[];
}

export interface Loadout {
    name: string;
    primaries: string[];
    secondaries: string[];
    selected_primary: number;
    selected_secondary: number;
}

export interface Weapon {
    id: string;
    name: string;
    // Add other weapon properties as needed
}

export interface Config {
    keybinds: {
        primary_weapon: string;
        secondary_weapon: string;
        alternative_fire: string;
    };
    mouse_config: {
        horizontal_multiplier: number;
        vertical_multiplier: number;
        acog_horizontal_multiplier: number;
        acog_vertical_multiplier: number;
        scroll_wheel_weapon_swap: boolean;
    };
}

export interface KeybindConfigOption {
    label: string;
    description: string;
    type: 'keybind' | 'checkbox' | 'slider' | 'char';
    key: string;
    value: any;
}

// Utility types
export type EventHandler<T = void> = () => T;
export type AsyncEventHandler<T = void> = () => Promise<T>;

// Component prop types
export interface WeaponCardProps {
    weaponId: string;
    weapon: Weapon | null;
    active: boolean;
    shooting: boolean;
}

export interface StatFieldProps {
    label: string;
    value: number;
    type: 'number';
    onChange: (value: number) => void;
}
