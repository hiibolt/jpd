<script lang="ts">
    import Titlebar from '../components/Titlebar.svelte';
    import GameCard from '../components/GameCard.svelte';
    import WeaponCard from '../components/WeaponCard.svelte';
    import ButtonPanel from '../components/ButtonPanel.svelte';
    import Background from '../components/Background.svelte';

    import {
        games,
        current_loadout_index, 
        current_weapon_index, 
        current_category_index, 
        current_game_index,
        shooting, 
        errors
    } from '../stores/state';
	import { clearErrors, restartApplication, changePrimaryWeapon, changeSecondaryWeapon } from '../lib/api';
	import { updateGridLayout } from '../lib/api';

    import { onMount, onDestroy } from 'svelte';
    import { tick } from 'svelte';

    // Reactive state derived from stores
    $: currentGame = $games[$current_game_index] ?? { 
        name: 'Game Not Found', 
        categories: [], 
        weapons: [] 
    };
    
    $: loadouts = currentGame.categories?.at($current_category_index)?.loadouts ?? [];
    
    $: currentLoadout = loadouts?.at($current_loadout_index) ?? { 
        name: 'Loadouts Not Found', 
        primaries: [], 
        secondaries: [], 
        selected_primary: 0, 
        selected_secondary: 0 
    };

    // Component state
    let leftColumnElement: HTMLElement;
    let resizeObserver: ResizeObserver | null = null;
    let layoutCalculationTimeout: number | null = null;

    // Debounced grid layout calculation
    function scheduleGridLayoutCalculation() {
        if (layoutCalculationTimeout) {
            clearTimeout(layoutCalculationTimeout);
        }
        layoutCalculationTimeout = setTimeout(calculateAndSendGridLayout, 100);
    }

    // Reactive effect to recalculate layout when loadouts change
    $: if (leftColumnElement && loadouts.length > 0) {
        // Use tick() to ensure DOM updates are complete before calculating
        tick().then(() => {
            scheduleGridLayoutCalculation();
        });
    }

    // Grid layout calculation with improved error handling
    function calculateAndSendGridLayout() {
        try {
            const activeCategory = leftColumnElement?.querySelector('.category-card.active');
            const loadoutGrid = activeCategory?.querySelector('.category-loadouts');
            
            if (!loadoutGrid) {
                console.log('No loadout grid found, using default');
                updateGridLayout(4);
                return;
            }

            // Wait for next frame to ensure layout is complete
            requestAnimationFrame(() => {
                try {
                    const loadoutCards = Array.from(loadoutGrid.querySelectorAll('.loadout-card'));
                    
                    if (loadoutCards.length === 0) {
                        console.log('No loadout cards found, using default');
                        updateGridLayout(4);
                        return;
                    }

                    let loadoutsPerRow = calculateLoadoutsPerRow(loadoutGrid, loadoutCards);
                    
                    // Clamp to reasonable bounds
                    loadoutsPerRow = Math.max(1, Math.min(loadoutsPerRow, 20));
                    
                    console.log(`Calculated loadouts per row: ${loadoutsPerRow} (from ${loadoutCards.length} total loadouts)`);
                    updateGridLayout(loadoutsPerRow);
                } catch (error) {
                    console.error('Error in grid layout calculation:', error);
                    updateGridLayout(4); // Fallback
                }
            });
        } catch (error) {
            console.error('Error in calculateAndSendGridLayout:', error);
            updateGridLayout(4); // Fallback
        }
    }

    // Separate function for calculating loadouts per row logic
    function calculateLoadoutsPerRow(loadoutGrid: Element, loadoutCards: Element[]): number {
        if (loadoutCards.length === 1) {
            return 1;
        }

        const gridStyle = window.getComputedStyle(loadoutGrid);
        const isFlexbox = gridStyle.display === 'flex';
        
        if (isFlexbox) {
            return calculateFlexboxLayout(loadoutGrid, loadoutCards, gridStyle);
        } else {
            return calculateGridLayout(loadoutCards, gridStyle);
        }
    }

    // Helper for flexbox layout calculation
    function calculateFlexboxLayout(container: Element, cards: Element[], style: CSSStyleDeclaration): number {
        const cardRects = cards.map(card => card.getBoundingClientRect());
        const firstRowY = cardRects[0].top;
        const tolerance = 5;
        
        let cardsInFirstRow = cardRects.filter(rect => 
            Math.abs(rect.top - firstRowY) <= tolerance
        ).length;
        
        // Fallback calculation if all cards appear to be on same row
        if (cardsInFirstRow === cards.length && cards.length > 1) {
            const containerWidth = container.getBoundingClientRect().width;
            const cardWidth = cardRects[0].width;
            const gap = parseFloat(style.gap) || parseFloat(style.columnGap) || 8;
            
            cardsInFirstRow = Math.max(1, Math.floor((containerWidth + gap) / (cardWidth + gap)));
        }
        
        return cardsInFirstRow;
    }

    // Helper for CSS Grid layout calculation
    function calculateGridLayout(cards: Element[], style: CSSStyleDeclaration): number {
        const gridColumns = style.gridTemplateColumns;
        
        if (gridColumns && gridColumns !== 'none') {
            return gridColumns.split(' ').length;
        }
        
        // Fallback to position-based calculation
        const cardRects = cards.map(card => card.getBoundingClientRect());
        const firstRowY = cardRects[0].top;
        
        return cardRects.filter(rect => Math.abs(rect.top - firstRowY) <= 5).length;
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

    onDestroy(() => {
        if (resizeObserver) {
            resizeObserver.disconnect();
        }
        if (layoutCalculationTimeout) {
            clearTimeout(layoutCalculationTimeout);
        }
    });

    // Event handlers
    const handleClearErrors = () => {
        clearErrors();
        console.log('Errors cleared');
    };

    const handleRestartApplication = () => {
        console.log('Restarting application...');
        restartApplication();
    };

    const handlePrimaryWeaponSelect = (index: number) => {
        changePrimaryWeapon(index);
    };

    const handleSecondaryWeaponSelect = (index: number) => {
        changeSecondaryWeapon(index);
    };

    const handleKeyDown = (event: KeyboardEvent, handler: () => void) => {
        if (event.key === 'Enter' || event.key === ' ') {
            event.preventDefault();
            handler();
        }
    };

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
		<div class="left-column card scrollable" bind:this={leftColumnElement}>
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
							<button class="btn btn-secondary" on:click={handleClearErrors}>
								Clear
							</button>
							<button class="btn btn-danger" on:click={handleRestartApplication}>
								Restart
							</button>
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
			<div class="card upper-right-card scrollable">
				<h3>{currentLoadout.name}</h3>
				
				<!-- Primary Weapons -->
				<div class="weapon-section">
					<h4 class="weapon-section-label">Primary Weapons</h4>
					{#if currentLoadout.primaries.length > 0}
						<div class="weapons-grid">
							{#each currentLoadout.primaries as id, i (id)}
								<div 
									class="weapon-selection"
									class:selected={i === currentLoadout.selected_primary}
									on:click={() => handlePrimaryWeaponSelect(i)}
									on:keydown={(e) => handleKeyDown(e, () => handlePrimaryWeaponSelect(i))}
									role="button"
									tabindex="0"
									aria-label="Select primary weapon {i + 1}"
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
							{#each currentLoadout.secondaries as id, i (id)}
								<div 
									class="weapon-selection"
									class:selected={i === currentLoadout.selected_secondary}
									on:click={() => handleSecondaryWeaponSelect(i)}
									on:keydown={(e) => handleKeyDown(e, () => handleSecondaryWeaponSelect(i))}
									role="button"
									tabindex="0"
									aria-label="Select secondary weapon {i + 1}"
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
  border-color: var(--accent);
  background: rgba(255, 255, 255, 0.05);
  transform: translateY(-2px);
  box-shadow: 0 4px 15px rgba(0, 0, 0, 0.2);
}

.weapon-selection.selected {
  border-color: var(--accent);
  background: rgba(0, 122, 204, 0.1);
  box-shadow: 0 0 15px rgba(0, 122, 204, 0.3);
}

.weapon-selection:focus {
  outline: none;
  border-color: var(--accent);
  box-shadow: 0 0 0 3px rgba(0, 122, 204, 0.2);
}
</style>

