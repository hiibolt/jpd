// src/lib/api.ts
import { invoke, Channel } from '@tauri-apps/api/core';
import {
    games,
    weapons,
    current_loadout_index,
    current_weapon_index,
    shooting,
    current_category_index,
    current_game_index
} from '../stores/state';

type SwitchedWeaponEvent = {
    event: 'SwitchedWeapon';
    data: { weapon_ind: number };
};
type StartedShootingEvent = {
    event: 'StartedShooting';
    data: { weapon_ind: number };
};
type StoppedShootingEvent = {
    event: 'StoppedShooting';
};

type Event = SwitchedWeaponEvent | StartedShootingEvent | StoppedShootingEvent;

let channel: Channel<Event>;

export async function initialize() {
    const loadedLoadouts = await invoke('get_games');
    games.set(loadedLoadouts as any);

    const loadedWeapons = await invoke('get_weapons');
    weapons.set(loadedWeapons as any);

    channel = new Channel<Event>();
    channel.onmessage = handleChannelEvent;

    await invoke('start_channel_reads', { channel });
}

function handleChannelEvent(message: Event) {
    switch (message.event) {
        case 'SwitchedWeapon':
            current_weapon_index.set(message.data.weapon_ind);
            break;
        case 'StartedShooting':
            shooting.set(true);
            current_weapon_index.set(message.data.weapon_ind);
            break;
        case 'StoppedShooting':
            shooting.set(false);
            break;
        default:
            console.warn('Unknown channel message', message);
    }
}

export async function changeGame(index: number) {
    const newIndex = await invoke('change_game', { newGameIndex: index });
    current_game_index.set(newIndex as number);
}
export async function changeCategory(index: number) {
    const newIndex = await invoke('change_category', { newCategoryIndex: index });
    current_category_index.set(newIndex as number);
}
export async function changeLoadout(index: number) {
    const newIndex = await invoke('change_loadout', { newLoadoutIndex: index });
    current_loadout_index.set(newIndex as number);
}