<script lang="ts">
    import Titlebar from '../components/Titlebar.svelte';
    import Banner from '../components/Banner.svelte';
    import ButtonPanel from '../components/ButtonPanel.svelte';
    import Background from '../components/Background.svelte';
    import ConfigGroup from '../components/ConfigGroup.svelte';
    import { config } from '../stores/state';

    let keybindConfigOptions: any[] = [
        { label: 'Primary Weapon', description: 'Switches to primary weapon', type: 'char', key: 'primary_weapon', value: $config.keybinds.primary_weapon },
		{ label: 'Secondary Weapon', description: 'Switches to secondary weapon', type: 'char', key: 'secondary_weapon', value: $config.keybinds.secondary_weapon },
		{ label: 'Alternative Fire', description: 'Bind your shoot key to this in-game for autofire to work', type: 'char', key: 'alternative_fire', value: $config.keybinds.alternative_fire }
    ];
</script>

<Background />
<main class="container">
    <Titlebar />
    <Banner />

    <div class="main-layout">
        <!-- Loadouts -->
        <div class="left-column card">
            <h2>Configuration Options</h2>
            <ConfigGroup configOptions={keybindConfigOptions} label="Keybinds" />
        </div>

        <!-- Active Loadout -->
        <div class="right-column">
            <div class="card upper-right-card">
                <h3>Settings</h3>
                <p>
                    Manage your settings and preferences.
                    <br>
                    <br>
                    Need help? Reach out to CLC and open a support ticket - we're here to help you play best.
                </p>
            </div>

            <ButtonPanel currentPage="settings"/>
        </div>
    </div>
</main>

<style>
:root {
  --fg: #222;
  --card-bg: rgba(255, 255, 255, 0.8);
  --card-shadow: 0 2px 10px rgba(0, 0, 0, 0.15);
  --accent: #0077ff;
  --shooting: #e53935;
  --not-shooting: #333;
  backdrop-filter: blur(20px);
}

@media (prefers-color-scheme: dark) {
  :root {
    --fg: #f6f6f6;
    --card-bg: rgba(30, 30, 30, 0.8);
    --card-shadow: 0 2px 10px rgba(0, 0, 0, 0.5);
  }
}

main.container {
  color: var(--fg);
  font-family: Inter, sans-serif;
  height: 100vh;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background: transparent; /* No background - acrylic look */
}

.main-layout {
  display: grid;
  grid-template-columns: 2fr 1fr;
  gap: 1rem;
  padding: 1rem;
  flex: 1;
  height: calc(100vh - 80px); /* titlebar + banner space */
  box-sizing: border-box;
  overflow: hidden;
}

.left-column {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(120px, 1fr));
  gap: 0.75rem;
  overflow-y: auto;
  padding: 0.5rem;
  text-align: center;
}

.right-column {
  display: flex;
  flex-direction: column;
  height: 100%;
  gap: 1rem;
}

.upper-right-card {
  flex: 9;
  display: flex;
  flex-direction: column;
  overflow-y: auto;
  text-align: center;
}
.upper-right-card p {
  margin: 0.5rem 0;
  line-height: 1.5;
  font-weight: 500;
}

.card {
  background-color: var(--card-bg);
  border-radius: 10px;
  box-shadow: var(--card-shadow);
  padding: 1rem;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

h3 {
  margin-top: 0;
}
</style>

