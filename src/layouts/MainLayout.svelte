<script lang="ts">
    import Titlebar from '../components/Titlebar.svelte';
    import Banner from '../components/Banner.svelte';
    import GameCard from '../components/GameCard.svelte';
    import WeaponCard from '../components/WeaponCard.svelte';
    import ButtonPanel from '../components/ButtonPanel.svelte';
    import Background from '../components/Background.svelte';

    import {
      games,
      current_loadout_index, current_weapon_index, current_category_index, current_game_index,
      shooting, 
      config,

      errors


    } from '../stores/state';
    import StatField from '../components/StatField.svelte';
	import { changeHorizontalMultiplier, changeVerticalMultiplier, clearErrors, restartApplication } from '../lib/api';


    $: currentGame = $games[$current_game_index] ?? { name: 'Game Not Found', categories: [], weapons: [] };
    $: loadouts = currentGame.categories?.at($current_category_index)?.loadouts ?? [];
    $: currentLoadout = loadouts?.at($current_loadout_index) ?? { name: 'Loadouts Not Found', weapon_ids: [] };
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
		{#if $errors.length > 0}
			<div class="card">
				<div class="errors-header">
					<h3>Errors</h3>
					<div class="error-buttons">
						<button class="error-btn clear-btn" on:click={() => {
							clearErrors();
							console.log('Errors cleared');
						}}>Clear</button>
						<button class="error-btn restart-btn" on:click={() => {
							console.log('Restarting application...');
							restartApplication();
						}}>Restart</button>
					</div>
				</div>
				<ul>
					{#each $errors as error}
						<div class="card error-item">
							<li>{error}</li>
						</div>
					{/each}
				</ul>
			</div>
		{/if}
		</div>

		<!-- Active Loadout -->
		<div class="right-column">
		<div class="card upper-right-card">
		<h3>{currentLoadout.name}</h3>
		{#each currentLoadout.weapon_ids as id, i}
		<WeaponCard
			weaponId={id}
			weapon={(currentGame.weapons ?? {})[id] ?? null}
			active={$current_weapon_index === i}
			shooting={$shooting && $current_weapon_index === i}
		/>
		{/each}
		</div>
		<div>
			<StatField
			label="Horizontal Sensitivity Multiplier"
			value={$config.mouse_config.horizontal_multiplier}
			type="number"
			onChange={(v) => {
				changeHorizontalMultiplier(v);
			}}
			/>
			<StatField
			label="Vertical Sensitivity Multiplier"
			value={$config.mouse_config.vertical_multiplier}
			type="number"
			onChange={(v) => {
				changeVerticalMultiplier(v);
			}}
			/>
		</div>
		<ButtonPanel currentPage="home"/>
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

.error-item {
  padding-left: 30px;
  color: red;
}

.errors-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 0.5rem;
}

.error-buttons {
  display: flex;
  gap: 0.5rem;
}

.error-btn {
  padding: 0.375rem 0.75rem;
  border: none;
  border-radius: 6px;
  font-size: 0.875rem;
  cursor: pointer;
  transition: background-color 0.2s;
}

.clear-btn {
  background-color: #6c757d;
  color: white;
}

.clear-btn:hover {
  background-color: #5a6268;
}

.restart-btn {
  background-color: #dc3545;
  color: white;
}

.restart-btn:hover {
  background-color: #c82333;
}

h3 {
  margin-top: 0;
}
</style>

