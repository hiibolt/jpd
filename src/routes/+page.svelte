
<main class="container">
  <!-- Tauri Titlebar -->
  <div data-tauri-drag-region class="titlebar">
    <button class="titlebar-button" aria-label="Minimize" onclick={minimize}>
      <img src="https://api.iconify.design/mdi:window-minimize.svg" alt="minimize" />
    </button>
    <button class="titlebar-button" aria-label="Maximize" onclick={maximize}>
      <img src="https://api.iconify.design/mdi:window-maximize.svg" alt="maximize" />
    </button>
    <button class="titlebar-button" aria-label="Close" onclick={close}>
      <img src="https://api.iconify.design/mdi:close.svg" alt="close" />
    </button>
  </div>

  <div class="banner">
    <h2>CLC JPD</h2>
  </div>

  <div class="main-layout">
    <!-- Left Column: Loadouts -->
    <div class="left-column card">
      {#if loadouts.length > 1}
        {#each loadouts as loadout, index}
          <button
            class="loadout-card"
            onclick={() => change_loadout(index)}
          >
            {loadout.name}
          </button>
        {/each}
      {:else}
        <p>Loading loadouts...</p>
      {/if}
    </div>

    <!-- Right Column: Active Loadout + Account Info -->
    <div class="right-column">
      <!-- Upper: Loadout Details -->
      <div class="card upper-right-card">
        {#if loadouts.length > 0}
          <h3>{loadouts[current_loadout_index].name}</h3>
          {#each loadouts[current_loadout_index].weapon_ids as item, ind}
            <div
              class="weapon-card {current_weapon_index === ind ? 'active open' : ''}"
            >
              <div class="weapon-header">
                <span>{item}</span>
                <span class="weapon-status {shooting && current_weapon_index === ind
                    ? 'shooting'
                    : 'not-shooting'}"></span>
              </div>

              <button
                class="weapon-card-toggle"
                onclick={() =>
                  document
                    .querySelectorAll('.weapon-card')
                    [ind]?.classList.toggle('open')
                }
              >
                {#if document.querySelectorAll('.weapon-card')[ind]?.classList.contains('open')}
                  Hide Details
                {:else}
                  Show Details
                {/if}
              </button>

              {#if weapons[item]}
                <div class="stats-group">
                  <p>Type: {weapons[item].type}</p>
                  {#if weapons[item].type === 'SingleFire'}
                    <p>Trigger Delay: {weapons[item].config.trigger_delay_ms} ms</p>
                    <p>Recoil Completion: {weapons[item].config.recoil_completion_ms} ms</p>
                    <p>Release Delay: {weapons[item].config.release_delay_ms} ms</p>
                    <p>DX/DY: {weapons[item].config.dx}/{weapons[item].config.dy}</p>
                    <p>Mag Size: {weapons[item].config.mag_size}</p>
                  {:else if weapons[item].type === 'FullAutoStandard'}
                    <p>RPM: {weapons[item].config.rpm}</p>
                    <p>First Shot Scale: {weapons[item].config.first_shot_scale}</p>
                    <p>Exponential Factor: {weapons[item].config.exponential_factor}</p>
                    <p>DX/DY: {weapons[item].config.dx}/{weapons[item].config.dy}</p>
                    <p>Mag Size: {weapons[item].config.mag_size}</p>
                  {/if}
                </div>
              {:else}
                <p>Weapon data unavailable</p>
              {/if}
            </div>
          {/each}
        {:else}
          <p>No loadout selected</p>
        {/if}
      </div>

      <!-- Lower: Account Panel -->
      <div class="card lower-right-card">
        <button data-tooltip="Settings">⚙️</button>
        <button data-tooltip="Account">👤</button>
        <div class="username">@me</div>
      </div>
    </div>
  </div>
</main>



<script lang="ts">
  import { writable } from 'svelte/store';
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
  
  type SingleFireConfig = {
    trigger_delay_ms: number;
    recoil_completion_ms: number;
    release_delay_ms: number;
    dx: number;
    dy: number;
    mag_size: number;
    autofire: boolean;
  };
  type FullAutoStandardConfig = {
    rpm: number;
    first_shot_scale: number;
    exponential_factor: number;
    dx: number;
    dy: number;
    mag_size: number;
  };
  type Weapon = {
    type: "SingleFire" | "FullAutoStandard";
    config: SingleFireConfig | FullAutoStandardConfig;
  };
  type Weapons = {
    [id: string]: Weapon;
  };

  let weapons = $state<Weapons>({});
  let loadouts = $state<Array<Loadout>>([]);
  let shooting = $state(false);
  let current_loadout_index = $state(0);
  let current_weapon_index = $state(0);

  const channel = new Channel<Event>(); 
  channel.onmessage = (message) => {
    switch (message.event) {
      case "StartedShooting":
        console.log(`Started shooting with weapon index: ${message.data.weapon_ind}`);
        shooting = true;
        current_weapon_index = message.data.weapon_ind;
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
    // Load loadouts
    loadouts = await invoke("get_loadouts");

    // Load weapons
    weapons = await invoke("get_weapons");

    await invoke("start_channel_reads", { channel });
  }
  load();

  async function change_loadout ( new_loadout_index: number ) {
    current_loadout_index = await invoke("change_loadout", { newLoadoutIndex: new_loadout_index });

    $inspect(current_loadout_index);
  }
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
