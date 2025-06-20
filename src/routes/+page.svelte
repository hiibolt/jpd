<main class="container">
  <!-- Tauri-based Titlebar -->
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

  <!-- Main content -->
  <div class="banner">
    <h2>CLC JPD</h2>
  </div>
  <div>
    <h3>Loadout: {loadout.name}</h3>
    <ul>
      {#each loadout.weapon_ids as item, ind}
        <li>
          <span class="weapon-name">{item} - {ind}</span>
          {#if shooting && weaponIndex === ind}
            <span class="shooting-indicator">Shooting!</span>
          {:else}
            <span class="not-shooting-indicator">Not Shooting</span>
          {/if}
        </li>
      {/each}
    </ul>
  </div>
</main>

<script lang="ts">
  import { invoke, Channel } from "@tauri-apps/api/core";
  import { getCurrentWindow } from '@tauri-apps/api/window';
  const appWindow = getCurrentWindow();

  type StartedShootingEvent = {
      event: "StartedShooting";
      data: {
        weapon_ind: number;
      };
    };
  type StoppedShootingEvent = {
      event: "StoppedShooting";
    };
  type Event = 
    | StartedShootingEvent
    | StoppedShootingEvent;
  type Loadout = {
    name: string;
    weapon_ids: Array<string>;
  }

  let loadout = $state({} as Loadout);
  let shooting = $state(false);
  let weaponIndex = $state(0);

  const channel = new Channel<Event>(); 
  channel.onmessage = (message) => {
    switch (message.event) {
      case "StartedShooting":
        console.log(`Started shooting with weapon index: ${message.data.weapon_ind}`);
        shooting = true;
        weaponIndex = message.data.weapon_ind;
        break;
      case "StoppedShooting":
        console.log("Stopped shooting");
        shooting = false;
        break;
      default:
        console.warn("Unknown event type:", message);
    }
  };

  // Load the loadout from the Rust backend
  async function load() {
    try {
      loadout = await invoke("get_loadout");
    } catch (error) {
      console.error("Failed to load loadout:", error);
    }
    await invoke("start_channel_reads", { channel });
  }
  load();

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
</script>

<style>
  /*
  .logo.vite:hover {
    filter: drop-shadow(0 0 2em #747bff);
  }
  .logo.svelte-kit:hover {
    filter: drop-shadow(0 0 2em #ff3e00);
  }
  */
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

  :root {
    font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
    font-size: 16px;
    font-weight: 400;

    background-color: #f6f6f6;

    font-synthesis: none;
    text-rendering: optimizeLegibility;
    -webkit-font-smoothing: antialiased;
    -moz-osx-font-smoothing: grayscale;
    -webkit-text-size-adjust: 100%;
  }

  .banner {
    display: flex;
    flex-direction: column;
    justify-content: center;
    text-align: center;
  }
  .container {
    padding-top: 3vh;
  }

  @media (prefers-color-scheme: dark) {
    :root {
      color: #f6f6f6;
      background-color: rgba(0, 0, 0, .15);   
      backdrop-filter: blur(5px);
    }
  }
</style>