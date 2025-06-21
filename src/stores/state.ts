// src/stores/state.ts
import { writable } from 'svelte/store';

export type Game = {
    name: string;
    categories: Category[];
    weapons: Weapons;
};
export type Category = {
    name: string;
    loadouts: Loadout[];
};
export type Loadout = {
    name: string;
    weapon_ids: string[];
};
export type SingleFireConfig = {
    name: string;
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
export const current_game_index = writable(0);
export const current_category_index = writable(0);
export const current_loadout_index = writable(0);
export const current_weapon_index = writable(0);
export const shooting = writable(false);
