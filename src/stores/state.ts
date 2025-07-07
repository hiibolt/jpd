// src/stores/state.ts
import { writable } from 'svelte/store';

type KeybindConfig = {
    require_right_hold: boolean;
    primary_weapon: string;
    secondary_weapon: string;
    alternative_fire: string;
};
type MouseConfig = {
    horizontal_multiplier: number;
    vertical_multiplier: number;
};
export type GlobalConfig = {
    keybinds: KeybindConfig;
    mouse_config: MouseConfig;
};

export type Game = {
    name: string;
    key?: string;
    key_status?: KeyStatus;
    categories?: Category[];
    weapons?: Weapons;
};

export type KeyStatus = 
    | { type: 'Invalid'; key: string }
    | { type: 'Valid'; key: string; timestamp: number }
    | { type: 'Expired'; key: string; timestamp: number }
    | { type: 'Banned'; key: string };
export type Category = {
    name: string;
    loadouts: Loadout[];
};
export type Loadout = {
    name: string;
    icon_url?: string;
    icon_only: boolean;
    weapon_ids: string[];
};
export type SingleFireConfig = {
    name: string;
    description?: string;
    trigger_delay_ms: number;
    recoil_completion_ms: number;
    release_delay_ms: number;
    dx: number;
    dy: number;
    mag_size: number;
    autofire: boolean;
};
export type FullAutoStandardConfig = {
    name: string;
    description?: string;
    rpm: number;
    first_shot_scale: number;
    exponential_factor: number;
    dx: number;
    dy: number;
    mag_size: number;
};

export type Weapon = {
    type: 'SingleFire' | 'FullAutoStandard';
    config: SingleFireConfig | FullAutoStandardConfig;
};

export type Weapons = Record<string, Weapon>;


// App-wide reactive state
export const games = writable<Game[]>([]);
export const config = writable<GlobalConfig>({
    keybinds: {
        require_right_hold: true,
        primary_weapon: '1',
        secondary_weapon: '2',
        alternative_fire: 'm',
    },
    mouse_config: {
        horizontal_multiplier: 1.0,
        vertical_multiplier: 1.0,
    },
});
export const current_game_index = writable(0);
export const current_category_index = writable(0);
export const current_loadout_index = writable(0);
export const current_weapon_index = writable(0);
export const shooting = writable(false);
export const errors = writable<string[]>([]);
export const version = writable<string>('?.?.?');