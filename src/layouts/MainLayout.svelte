<script lang="ts">
    import Titlebar from '../components/Titlebar.svelte';
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
	import { clearErrors, restartApplication, changePrimaryWeapon, changeSecondaryWeapon } from '../lib/api';
	import { updateGridLayout } from '../lib/api';

    import { onMount, afterUpdate, onDestroy } from 'svelte';

    let leftColumnElement: HTMLElement;
    let resizeObserver: ResizeObserver | null = null;
    let layoutCalculationTimeout: number | null = null;

    $: currentGame = $games[$current_game_index] ?? { name: 'Game Not Found', categories: [], weapons: [] };
    $: loadouts = currentGame.categories?.at($current_category_index)?.loadouts ?? [];
    $: currentLoadout = loadouts?.at($current_loadout_index) ?? { name: 'Loadouts Not Found', primaries: [], secondaries: [], selected_primary: 0, selected_secondary: 0 };

    // Debounced grid layout calculation
    function scheduleGridLayoutCalculation() {
        if (layoutCalculationTimeout) {
            clearTimeout(layoutCalculationTimeout);
        }
        layoutCalculationTimeout = setTimeout(calculateAndSendGridLayout, 100);
    }

    // Calculate and send grid layout to backend when loadouts change
    $: if (leftColumnElement && loadouts.length > 0) {
        scheduleGridLayoutCalculation();
    }

    function calculateAndSendGridLayout() {
        // Look for the loadout grid within the active category
        const activeCategory = leftColumnElement?.querySelector('.category-card.active');
        const loadoutGrid = activeCategory?.querySelector('.category-loadouts');
        
        if (!loadoutGrid) {
            console.log('No loadout grid found, using default');
            updateGridLayout(4);
            return;
        }

        // Wait for next frame to ensure layout is complete
        requestAnimationFrame(() => {
            const loadoutCards = Array.from(loadoutGrid.querySelectorAll('.loadout-card'));
            
            if (loadoutCards.length === 0) {
                console.log('No loadout cards found, using default');
                updateGridLayout(4);
                return;
            }

            let loadoutsPerRow = 1;

            if (loadoutCards.length === 1) {
                loadoutsPerRow = 1;
            } else {
                // Get computed style for the container
                const gridStyle = window.getComputedStyle(loadoutGrid);
                
                // Use a more robust method to determine grid layout
                // Check if it's using flexbox
                const isFlexbox = gridStyle.display === 'flex';
                
                if (isFlexbox) {
                    // For flexbox with flex-wrap, determine row count by comparing Y positions
                    const cardRects = loadoutCards.map(card => card.getBoundingClientRect());
                    const firstRowY = cardRects[0].top;
                    const tolerance = 5; // Allow small differences due to rounding
                    
                    // Count cards in the first row by checking Y position
                    loadoutsPerRow = cardRects.filter(rect => 
                        Math.abs(rect.top - firstRowY) <= tolerance
                    ).length;
                    
                    // Fallback: if all cards appear to be on same row, calculate based on width
                    if (loadoutsPerRow === loadoutCards.length && loadoutCards.length > 1) {
                        const containerWidth = loadoutGrid.getBoundingClientRect().width;
                        const cardWidth = cardRects[0].width;
                        
                        // Get actual gap from computed style
                        const gap = parseFloat(gridStyle.gap) || parseFloat(gridStyle.columnGap) || 8;
                        
                        loadoutsPerRow = Math.max(1, Math.floor((containerWidth + gap) / (cardWidth + gap)));
                    }
                } else {
                    // For CSS Grid, use grid-template-columns
                    const gridColumns = gridStyle.gridTemplateColumns;
                    if (gridColumns && gridColumns !== 'none') {
                        // Count the number of columns defined
                        loadoutsPerRow = gridColumns.split(' ').length;
                    } else {
                        // Fallback to position-based calculation
                        const cardRects = loadoutCards.map(card => card.getBoundingClientRect());
                        const firstRowY = cardRects[0].top;
                        loadoutsPerRow = cardRects.filter(rect => 
                            Math.abs(rect.top - firstRowY) <= 5
                        ).length;
                    }
                }
            }

            // Clamp to reasonable bounds
            loadoutsPerRow = Math.max(1, Math.min(loadoutsPerRow, 20));
            
            // Get computed style for debug logging
            const gridStyle = window.getComputedStyle(loadoutGrid);
            
            console.log(`Calculated loadouts per row: ${loadoutsPerRow} (from ${loadoutCards.length} total loadouts)`);
            console.log(`Grid display: ${gridStyle.display}, flex-wrap: ${gridStyle.flexWrap}`);
            console.log(`Container width: ${loadoutGrid.getBoundingClientRect().width}px`);
            
            updateGridLayout(loadoutsPerRow);
        });
    }

    onMount(() => {
        // Set up ResizeObserver to recalculate when container size changes
        if (typeof ResizeObserver !== 'undefined') {
            resizeObserver = new ResizeObserver(() => {
                scheduleGridLayoutCalculation();
            });
        }
        
        // Initial calculation with a small delay for rendering
        setTimeout(calculateAndSendGridLayout, 150);
    });

    // Recalculate when the DOM updates (category/loadout changes)
    afterUpdate(() => {
        if (leftColumnElement && loadouts.length > 0) {
            scheduleGridLayoutCalculation();
        }
    });

    onDestroy(() => {
        if (resizeObserver) {
            resizeObserver.disconnect();
        }
        if (layoutCalculationTimeout) {
            clearTimeout(layoutCalculationTimeout);
        }
    });

    // Observe the left column for size changes
    $: if (leftColumnElement && resizeObserver) {
        resizeObserver.observe(leftColumnElement);
    }
</script>

<Background />
<main class="container">
	<Titlebar />
	<br>

	<div class="main-layout">
		<!-- Loadouts -->
		<div class="left-column card" bind:this={leftColumnElement}>
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
				
				<!-- Primary Weapons -->
				<div class="weapon-section">
					<h4 class="weapon-section-label">Primary Weapons</h4>
					{#if currentLoadout.primaries.length > 0}
						<div class="weapons-grid">
							{#each currentLoadout.primaries as id, i}
							<div 
								class="weapon-selection {i === currentLoadout.selected_primary ? 'selected' : ''}"
								on:click={() => changePrimaryWeapon(i)}
								role="button"
								tabindex="0"
								on:keydown={(e) => e.key === 'Enter' && changePrimaryWeapon(i)}
							>
								<WeaponCard
								weaponId={id}
								weapon={(currentGame.weapons ?? {})[id] ?? null}
								active={$current_weapon_index === 0 && i === currentLoadout.selected_primary}
								shooting={$shooting && $current_weapon_index === 0 && i === currentLoadout.selected_primary}
								/>
							</div>
							{/each}
						</div>
					{:else}
						<p>No primary weapons</p>
					{/if}
				</div>

				<!-- Secondary Weapons -->
				<div class="weapon-section">
				<h4 class="weapon-section-label">Secondary Weapons</h4>
				{#if currentLoadout.secondaries.length > 0}
					<div class="weapons-grid">
					{#each currentLoadout.secondaries as id, i}
					<div 
						class="weapon-selection {i === currentLoadout.selected_secondary ? 'selected' : ''}"
						on:click={() => changeSecondaryWeapon(i)}
						role="button"
						tabindex="0"
						on:keydown={(e) => e.key === 'Enter' && changeSecondaryWeapon(i)}
					>
						<WeaponCard
						weaponId={id}
						weapon={(currentGame.weapons ?? {})[id] ?? null}
						active={$current_weapon_index === 1 && i === currentLoadout.selected_secondary}
						shooting={$shooting && $current_weapon_index === 1 && i === currentLoadout.selected_secondary}
						/>
					</div>
					{/each}
					</div>
				{:else}
					<p>No secondary weapons</p>
				{/if}
				</div>
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
  --accent: #bf0f70;
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
  font-family: Tahoma, sans-serif;
  height: 100vh;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background: transparent; /* No background - acrylic look */
}

.main-layout {
  display: grid;
  grid-template-columns: 2fr 1fr;
  gap: 0.75rem;
  padding: 0.75rem;
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
  min-height: 0;
  gap: 1rem;
}

.upper-right-card {
  flex: 9;
  display: flex;
  flex-direction: column;
  overflow-y: auto;
   /* Allow flex item to shrink below content size */
  max-height: 100%; /* Ensure it doesn't exceed container */
  height: 0; /* Force the flex item to use flex sizing */
}

.card {
  background-color: var(--card-bg);
  border-radius: 10px;
  box-shadow: var(--card-shadow);
  padding: 1rem;
  display: flex;
  flex-direction: column;
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

.weapon-section {
  margin: 1rem 0;
}

.weapon-section-label {
  margin: 0 0 0.5rem 0;
  color: var(--fg);
  font-size: 1rem;
  font-weight: 600;
  opacity: 0.9;
}

.weapons-grid {
  display: flex;
  flex-wrap: wrap;
  margin-top: 0.5rem;
}

.weapon-selection {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 0.4rem;
  border-radius: 10px;
  border: 2px solid transparent;
  transition: all 0.2s ease;
  cursor: pointer;
  background: rgba(255, 255, 255, 0.02);
}

.weapon-selection:hover {
  border-color: var(--accent, var(--accent));
  background: rgba(255, 255, 255, 0.05);
  transform: translateY(-2px);
  box-shadow: 0 4px 15px rgba(0, 0, 0, 0.2);
}

.weapon-selection.selected {
  border-color: var(--accent, var(--accent));
  background: rgba(0, 122, 204, 0.1);
  box-shadow: 0 0 15px rgba(0, 122, 204, 0.3);
}

.weapon-selection:focus {
  outline: none;
  border-color: var(--accent, var(--accent));
  box-shadow: 0 0 0 3px rgba(0, 122, 204, 0.2);
}

/* Custom scrollbar styling for right column */
.upper-right-card::-webkit-scrollbar {
  width: 8px;
}

.upper-right-card::-webkit-scrollbar-track {
  background: rgba(255, 255, 255, 0.1);
  border-radius: 4px;
}

.upper-right-card::-webkit-scrollbar-thumb {
  background: var(--accent, var(--accent));
  border-radius: 4px;
  opacity: 0.7;
}

.upper-right-card::-webkit-scrollbar-thumb:hover {
  background: #0056cc;
  opacity: 1;
}

/* Firefox scrollbar styling - separate from WebKit */
@-moz-document url-prefix() {
  .upper-right-card {
    scrollbar-width: thin;
    scrollbar-color: var(--accent, var(--accent)) rgba(255, 255, 255, 0.1);
  }
}

@media (prefers-color-scheme: dark) {
  .upper-right-card::-webkit-scrollbar-track {
    background: rgba(255, 255, 255, 0.05);
  }
  
  @-moz-document url-prefix() {
    .upper-right-card {
      scrollbar-color: var(--accent, var(--accent)) rgba(255, 255, 255, 0.05);
    }
  }
}
</style>

