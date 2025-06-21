<script lang="ts">
    import Titlebar from '../components/Titlebar.svelte';
    import Banner from '../components/Banner.svelte';
    import GameCard from '../components/GameCard.svelte';
    import LoadoutCard from '../components/LoadoutCard.svelte';
    import WeaponCard from '../components/WeaponCard.svelte';
    import AccountPanel from '../components/AccountPanel.svelte';
    import Background from '../components/Background.svelte';

    import {
      games,
      current_loadout_index, current_weapon_index, current_category_index, current_game_index,
      shooting 
    } from '../stores/state';

    $: currentGame = $games[$current_game_index] ?? { name: 'Loading...', categories: [], weapons: [] };
    $: loadouts = currentGame.categories[$current_category_index]?.loadouts ?? [];
    $: currentLoadout = loadouts[$current_loadout_index] ?? { name: 'Loading...', weapon_ids: [] };
</script>

<Background />
<main class="container">
    <Titlebar />
    <Banner />

    <div class="main-layout">
        <!-- Loadouts -->
        <div class="left-column card">
        {#if $games.length > 0}
          {#each $games as game, index}
            <GameCard game={game} index={index} />
          {/each}
        {:else}
            <h2>Loading Games...</h2>
        {/if}
        </div>

        <!-- Active Loadout -->
        <div class="right-column">
        <div class="card upper-right-card">
            <h3>{currentLoadout.name}</h3>
            {#each currentLoadout.weapon_ids as id, i}
            <WeaponCard
                id={id}
                data={currentGame.weapons[id] ?? null}
                active={$current_weapon_index === i}
                shooting={$shooting && $current_weapon_index === i}
            />
            {/each}
        </div>

        <AccountPanel />
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

