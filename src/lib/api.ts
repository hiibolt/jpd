// src/lib/api.ts
import { invoke, Channel } from '@tauri-apps/api/core';
import {
    loadouts,
    weapons,
    current_loadout_index,
    current_weapon_index,
    shooting
} from '../stores/state';

type StartedShootingEvent = {
    event: 'StartedShooting';
    data: { weapon_ind: number };
};

type StoppedShootingEvent = {
    event: 'StoppedShooting';
};

type Event = StartedShootingEvent | StoppedShootingEvent;

let channel: Channel<Event>;

export async function initialize() {
    const loadedLoadouts = await invoke('get_loadouts');
    loadouts.set(loadedLoadouts as any);

    const loadedWeapons = await invoke('get_weapons');
    weapons.set(loadedWeapons as any);

    channel = new Channel<Event>();
    channel.onmessage = handleChannelEvent;

    await invoke('start_channel_reads', { channel });
}

function handleChannelEvent(message: Event) {
    switch (message.event) {
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

export async function changeLoadout(index: number) {
    const newIndex = await invoke('change_loadout', { newLoadoutIndex: index });
    current_loadout_index.set(newIndex as number);
}
