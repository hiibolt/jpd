// src/lib/api.ts
import { invoke, Channel } from '@tauri-apps/api/core';
import {
    games,
    current_loadout_index,
    current_weapon_index,
    shooting,
    current_category_index,
    current_game_index,
    config,
    type GlobalConfig,
    type Game
} from '../stores/state';

type UpdatedGamesEvent = {
    event: 'UpdatedGames';
    data: { games: Game[]; };
};
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
type Event = UpdatedGamesEvent | SwitchedWeaponEvent | StartedShootingEvent | StoppedShootingEvent;

let channel: Channel<Event>;

export async function initialize() {
    const loadedLoadouts = await invoke('get_games');
    games.set(loadedLoadouts as any);
    console.log('Games loaded:', loadedLoadouts);

    const loadedConfig = await invoke('get_config');
    config.set(loadedConfig as any);
    console.log('Config loaded:', loadedConfig);

    channel = new Channel<Event>();
    channel.onmessage = handleChannelEvent;
    await invoke('start_channel_reads', { channel });
}

function handleChannelEvent(message: Event) {
    switch (message.event) {
        case 'UpdatedGames':
            games.set(message.data.games as any);
            console.log('Games updated:', message.data.games);
            break;
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
export async function changeHorizontalMultiplier(newMultiplier: number) {
    const new_config = await invoke('change_horizontal_multiplier', { newMultiplier });
    config.set(new_config as any);
}
export async function changeVerticalMultiplier(newMultiplier: number) {
    const new_config = await invoke('change_vertical_multiplier', { newMultiplier });
    config.set(new_config as any);
}
export async function changeSetting(setting: string, value: string | boolean | number) {
    const new_config = await invoke('change_setting', { setting, value });
    config.set(new_config as any);
}