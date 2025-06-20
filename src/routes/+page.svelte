<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWindow } from '@tauri-apps/api/window';

  console.log("Tauri + SvelteKit app started :3");

  // when using `"withGlobalTauri": true`, you may use
  // const { getCurrentWindow } = window.__TAURI__.window;

  const appWindow = getCurrentWindow();

  function minimize() {
    console.log("Minimizing window");
    appWindow.minimize();
  }
  function maximize() {
    console.log("Maximizing window");
    appWindow.toggleMaximize();
  }
  function close() {
    console.log("Closing window");
    appWindow.close();
  }
  
  type Loadout = {
    name: string;
    items: Array<string>;
  }

  let loadout = $state({} as Loadout);
  let name = $state("");
  let greetMsg = $state("");

  // Load the loadout from the Rust backend
  async function loadLoadout() {
    try {
      loadout = await invoke("get_loadout");
    } catch (error) {
      console.error("Failed to load loadout:", error);
    }
  }
  loadLoadout();

  async function greet(event: Event) {
    event.preventDefault();
    // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
    greetMsg = await invoke("greet", { name });
  }
</script>

<main class="container">
  <style>
    .titlebar {
      height: 30px;
      user-select: none;
      display: flex;
      margin-right: 5px;
      justify-content: flex-end;
      position: fixed;
      top: 0;
      left: 0;
      right: 0;
    }
    .titlebar-button {
      border-radius: 8px;
      border: 1px solid transparent;
      padding: 0.6em 1.2em;

      background-color: transparent;
      display: inline-flex;
      margin: 5px 0px 0px 5px;
      justify-content: center;
      align-items: center;
      width: 20px;
      height: 25px;
      user-select: none;
      -webkit-user-select: none;
    }
    .titlebar-button:hover {
      transition: background-color 0.20s ease-in-out;
      background: rgba(255, 255, 255, 0.1);
    }
  </style>
  <div data-tauri-drag-region class="titlebar">
    <button class="titlebar-button" id="titlebar-minimize" aria-label="Minimize" onclick={minimize}>
      <img
      src="https://api.iconify.design/mdi:window-minimize.svg"
      alt="minimize"
      />
    </button>
    <button class="titlebar-button" id="titlebar-maximize" aria-label="Maximize" onclick={maximize}>
      <img
      src="https://api.iconify.design/mdi:window-maximize.svg"
      alt="maximize"
      />
    </button>
    <button class="titlebar-button" id="titlebar-close" aria-label="Close" onclick={close}>
      <img src="https://api.iconify.design/mdi:close.svg" alt="close" />
    </button>
  </div>
  <h1>Welcome to Tauri + Svelte</h1>
  <h2>Loadout: {loadout.name}</h2>
</main>

<style>
  /*
  .logo.vite:hover {
    filter: drop-shadow(0 0 2em #747bff);
  }
  .logo.svelte-kit:hover {
    filter: drop-shadow(0 0 2em #ff3e00);
  }
  */

:root {
  font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
  font-size: 16px;
  line-height: 24px;
  font-weight: 400;

  background-color: #f6f6f6;

  font-synthesis: none;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  -webkit-text-size-adjust: 100%;
}

.container {
  margin: 0;
  padding-top: 10vh;
  display: flex;
  flex-direction: column;
  justify-content: center;
  text-align: center;
}

h1 {
  text-align: center;
}

/*
input,
button {
  border-radius: 8px;
  border: 1px solid transparent;
  padding: 0.6em 1.2em;
  font-size: 1em;
  font-weight: 500;
  font-family: inherit;
  color: #0f0f0f;
  background-color: #ffffff;
  transition: border-color 0.25s;
  box-shadow: 0 2px 2px rgba(0, 0, 0, 0.2);
}
*/

@media (prefers-color-scheme: dark) {
  :root {
    color: #f6f6f6;
    background-color: rgba(0, 0, 0, .15);   
    backdrop-filter: blur(5px);
  }
}

</style>
