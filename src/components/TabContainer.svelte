<script lang="ts">
    import { createEventDispatcher } from 'svelte';
    
    export let tabs: Array<{ id: string; label: string; icon?: string }> = [];
    export let activeTab: string = '';
    
    const dispatch = createEventDispatcher<{ tabChange: string }>();
    
    const handleTabClick = (tabId: string) => {
        dispatch('tabChange', tabId);
    };
    
    const handleKeyDown = (event: KeyboardEvent, tabId: string) => {
        if (event.key === 'Enter' || event.key === ' ') {
            event.preventDefault();
            handleTabClick(tabId);
        }
        
        // Arrow key navigation
        if (event.key === 'ArrowLeft' || event.key === 'ArrowRight') {
            event.preventDefault();
            const currentIndex = tabs.findIndex(tab => tab.id === activeTab);
            let newIndex;
            
            if (event.key === 'ArrowLeft') {
                newIndex = currentIndex > 0 ? currentIndex - 1 : tabs.length - 1;
            } else {
                newIndex = currentIndex < tabs.length - 1 ? currentIndex + 1 : 0;
            }
            
            handleTabClick(tabs[newIndex].id);
        }
    };
</script>

<div class="tabs-container" role="tablist" aria-label="Settings navigation">
    {#each tabs as tab, index}
        <button
            class="tab"
            class:active={activeTab === tab.id}
            role="tab"
            tabindex={activeTab === tab.id ? 0 : -1}
            aria-selected={activeTab === tab.id}
            aria-controls="tabpanel-{tab.id}"
            id="tab-{tab.id}"
            on:click={() => handleTabClick(tab.id)}
            on:keydown={(e) => handleKeyDown(e, tab.id)}
        >
            {#if tab.icon}
                <span class="tab-icon" aria-hidden="true">{tab.icon}</span>
            {/if}
            <span class="tab-label">{tab.label}</span>
        </button>
    {/each}
</div>

<style>
.tabs-container {
    display: flex;
    background-color: rgba(255, 255, 255, 0.05);
    border-radius: 12px;
    padding: 0.25rem;
    margin-bottom: 1.5rem;
    border: 1px solid rgba(255, 255, 255, 0.1);
    backdrop-filter: blur(10px);
    gap: 0.25rem;
}

.tab {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
    padding: 0.75rem 1rem;
    border: none;
    border-radius: 8px;
    background: transparent;
    color: var(--fg);
    cursor: pointer;
    transition: all 0.2s ease;
    font-size: 0.9rem;
    font-weight: 500;
    opacity: 0.7;
    position: relative;
}

.tab:hover {
    opacity: 1;
    background-color: rgba(255, 255, 255, 0.05);
    transform: translateY(-1px);
}

.tab:focus {
    outline: none;
    box-shadow: 0 0 0 2px var(--accent);
    opacity: 1;
}

.tab.active {
    background-color: var(--accent);
    color: white;
    opacity: 1;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.2);
}

.tab.active:hover {
    background-color: var(--accent);
    transform: translateY(-1px);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
}

.tab-icon {
    font-size: 1.1rem;
    line-height: 1;
}

.tab-label {
    font-weight: 600;
}

@media (max-width: 480px) {
    .tabs-container {
        flex-direction: column;
        gap: 0.25rem;
    }
    
    .tab {
        justify-content: flex-start;
        padding: 0.6rem 1rem;
    }
}
</style>
