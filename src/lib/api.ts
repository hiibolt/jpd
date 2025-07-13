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
    type Game,
    errors,
    version
} from '../stores/state';

type UpdatedGamesEvent = {
    event: 'UpdatedGames';
    data: { games: Game[]; };
};
type SwitchedWeaponEvent = {
    event: 'SwitchedWeapon';
    data: { weapon_ind: number };
};
type SwitchedLoadoutEvent = {
    event: 'SwitchedLoadout';
    data: { loadout_ind: number };
};
type SwitchedCategoryEvent = {
    event: 'SwitchedCategory';
    data: { category_ind: number };
};
type StartedShootingEvent = {
    event: 'StartedShooting';
    data: { weapon_ind: number };
};
type StoppedShootingEvent = {
    event: 'StoppedShooting';
};
type Event = UpdatedGamesEvent | SwitchedWeaponEvent | SwitchedLoadoutEvent | SwitchedCategoryEvent | StartedShootingEvent | StoppedShootingEvent;

let channel: Channel<Event>;

export async function initialize() {
    try {
        console.log('Initializing application...');
        version.set(await invoke('get_version') as string);
        console.log('Version:', version);

        await invoke('load_games_wrapper');
        console.log('Games loaded successfully');

        const loadedLoadouts = await invoke('get_games');
        games.set(loadedLoadouts as any);
        console.log('Games loaded:', loadedLoadouts);

        const loadedConfig = await invoke('get_config');
        config.set(loadedConfig as any);
        console.log('Config loaded:', loadedConfig);

        channel = new Channel<Event>();
        channel.onmessage = handleChannelEvent;
        await invoke('start_channel_reads', { channel });
    } catch (error: any) {
        handleError('Initialization failed', error);
    }
}

function handleError(context: string, error: any) {
    console.error(`${context}: `, error);

    // Push the error message to the errors store
    errors.update((currentErrors) => [...currentErrors, String(error)]);
}
function handleChannelEvent(message: Event) {
    console.log('Received channel event:', message); // Debug log to see what we're receiving
    
    switch (message.event) {
        case 'UpdatedGames':
            games.set(message.data.games as any);
            console.log('Games updated:', message.data.games);
            break;
        case 'SwitchedWeapon':
            current_weapon_index.set(message.data.weapon_ind);
            console.log('Weapon switched to index:', message.data.weapon_ind);
            break;
        case 'SwitchedLoadout':
            current_loadout_index.set(message.data.loadout_ind);
            console.log('Loadout switched to index:', message.data.loadout_ind);
            break;
        case 'SwitchedCategory':
            current_category_index.set(message.data.category_ind);
            console.log('Category switched to index:', message.data.category_ind);
            break;
        case 'StartedShooting':
            shooting.set(true);
            current_weapon_index.set(message.data.weapon_ind);
            break;
        case 'StoppedShooting':
            shooting.set(false);
            break;
        default:
            console.warn('Unknown channel message:', message);
            console.warn('Event type received:', typeof (message as any).event, (message as any).event);
            console.warn('Full message structure:', JSON.stringify(message, null, 2));
    }
}
export function changeGame(index: number) {
    invoke('change_game', { newGameIndex: index })
        .then((newIndex) => current_game_index.set(newIndex as number))
        .catch((error) => handleError('Change to game failed', error));
}
export function changeCategory(index: number) {
    invoke('change_category', { newCategoryIndex: index })
        .then((newIndex) => current_category_index.set(newIndex as number))
        .catch((error) => handleError('Change to category failed', error));
}
export function changeLoadout(index: number) {
    invoke('change_loadout', { newLoadoutIndex: index })
        .then((newIndex) => current_loadout_index.set(newIndex as number))
        .catch((error) => handleError('Change to loadout failed', error));
}
export function changePrimaryWeapon(index: number) {
    invoke('change_primary_weapon', { newPrimaryIndex: index })
        .then((new_games) => games.set(new_games as Game[]))
        .catch((error) => handleError('Change primary weapon failed', error));
}
export function changeSecondaryWeapon(index: number) {
    invoke('change_secondary_weapon', { newSecondaryIndex: index })
        .then((new_games) => games.set(new_games as Game[]))
        .catch((error) => handleError('Change secondary weapon failed', error));
}
export function changeHorizontalMultiplier(newMultiplier: number) {
    invoke('change_horizontal_multiplier', { newMultiplier })
        .then((new_config) => config.set(new_config as any))
        .catch((error) => handleError('Change to horizontal multiplier failed', error));
}
export function changeVerticalMultiplier(newMultiplier: number) {
    invoke('change_vertical_multiplier', { newMultiplier })
        .then((new_config) => config.set(new_config as any))
        .catch((error) => handleError('Change to vertical multiplier failed', error));
}
export function changeAcogHorizontalMultiplier(newMultiplier: number) {
    invoke('change_acog_horizontal_multiplier', { newMultiplier })
        .then((new_config) => config.set(new_config as any))
        .catch((error) => handleError('Change to ACOG horizontal multiplier failed', error));
}
export function changeAcogVerticalMultiplier(newMultiplier: number) {
    invoke('change_acog_vertical_multiplier', { newMultiplier })
        .then((new_config) => config.set(new_config as any))
        .catch((error) => handleError('Change to ACOG vertical multiplier failed', error));
}
export function changeSetting(setting: string, value: string | boolean | number) {
    invoke('change_setting', { setting, value })
        .then((new_config) => config.set(new_config as any))
        .catch((error) => handleError('Change to settings failed', error));
}
export async function setWeaponConfig ( weaponId: string, field: string, newValue: any ) {
    invoke('set_weapon_config', { weaponId, field, newValue })
        .then((new_games) => games.set(new_games as any))
        .catch((error) => handleError('Set weapon config failed', error));
}

export async function submitGameKey(gameName: string, key: string): Promise<Game[] | null> {
    try {
        const updatedGames = await invoke('submit_game_key', { gameName, key });
        games.set(updatedGames as Game[]);
        return updatedGames as Game[];
    } catch (error) {
        handleError('Submit game key failed', error);
        return null;
    }
}

export function clearErrors() {
    errors.set([]);
}

export async function resetConfigFromServer() {
    try {
        console.log('Resetting configuration from server...');
        const result = await invoke('reset_config_from_server');
        const newGames = result as Game[];
        
        // Update stores with fresh data
        games.set(newGames);
        
        console.log('Configuration reset successfully:', newGames);
    } catch (error) {
        handleError('Reset configuration failed', error);
    }
}

export async function restartApplication() {
    try {
        await invoke('restart_app');
    } catch (error) {
        handleError('Restart application failed', error);
    }
}

export async function updateGridLayout(loadoutsPerRow: number) {
    try {
        await invoke('update_grid_layout', {
            loadoutsPerRow
        });
    } catch (error) {
        handleError('Update grid layout failed', error);
    }
}