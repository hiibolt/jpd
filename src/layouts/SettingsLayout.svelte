<script lang="ts">
    import Titlebar from '../components/Titlebar.svelte';
    import ButtonPanel from '../components/ButtonPanel.svelte';
    import Background from '../components/Background.svelte';
    import TabContainer from '../components/tabs/TabContainer.svelte';
    import KeybindsTab from '../components/tabs/KeybindsTab.svelte';
    import VisualsTab from '../components/tabs/VisualsTab.svelte';
    import DangerZoneTab from '../components/tabs/DangerZoneTab.svelte';
	import { open } from '@tauri-apps/plugin-shell';

    // Tab configuration
    const tabs = [
        { id: 'keybinds', label: 'Keybinds', icon: '⌨️' },
        { id: 'visuals', label: 'Visuals', icon: '🎨' },
        { id: 'danger', label: 'Danger Zone', icon: '⚠️' }
    ];
    
    let activeTab = 'keybinds';

    // Event handlers
    const handleTabChange = (tabId: string) => {
        activeTab = tabId;
    };

    const handleOpenDiscord = () => {
        open("https://discord.gg/pulldown");
    };
	const handleOpenGithub = () => {
        open("https://github.com/hiibolt");
    };
</script>

<Background />
<main class="container">
    <Titlebar />
	<br>

    <div class="main-layout">
        <!-- Settings Navigation and Content -->
        <div class="left-column card scrollable">
            
            <!-- Tab Navigation -->
            <TabContainer 
                {tabs} 
                {activeTab} 
                on:tabChange={(e) => handleTabChange(e.detail)} 
            />
            
            <!-- Tab Content -->
            {#if activeTab === 'keybinds'}
                <KeybindsTab />
            {:else if activeTab === 'visuals'}
                <VisualsTab />
            {:else if activeTab === 'danger'}
                <DangerZoneTab />
            {/if}
        </div>

        <!-- Active Loadout -->
        <div class="right-column">	
            <div class="card upper-right-card scrollable">
                <h3>Settings</h3>
                <p>
                    Manage your settings and preferences.
                    <br>
                    <br>
                    Need help? Reach out to JPD staff in the 
                    <button class="link-button" on:click={handleOpenDiscord}>
                        Discord
                    </button> 
                    and open a support ticket - we're here to help you play best.
                </p>
				<div class="username">
					Maintained by <button class="link-button" on:click={handleOpenGithub}>
                        @hiibolt
                    </button>  with &lt;3
				</div>
            </div>

            <ButtonPanel currentPage="settings"/>
        </div>
    </div>
</main>

<style>
.left-column {
  text-align: center;
}

.upper-right-card {
  text-align: center;
}

.upper-right-card p {
  margin: 0.5rem 0;
  line-height: 1.5;
  font-weight: 500;
}

.username {
  font-size: 0.85em;
  opacity: 0.75;
}
</style>

