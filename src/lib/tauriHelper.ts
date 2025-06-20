// src/lib/tauriHelpers.ts
import { getCurrentWindow } from '@tauri-apps/api/window';

const appWindow = getCurrentWindow();

export function minimize() {
    console.log('Minimizing window');
    appWindow.minimize();
}

export function maximize() {
    console.log('Toggling maximize');
    appWindow.toggleMaximize();
}

export function close() {
    console.log('Closing window');
    appWindow.close();
}
