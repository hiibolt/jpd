<script lang="ts">
    import { checkForUpdates, performUpdate, exitApp } from '../lib/api';
    import { onMount } from 'svelte';

    export let visible = false;
    let isUpdating = false;
    let updateError = '';

    onMount(async () => {
        // Check for updates when the component mounts
        try {
            const hasUpdate = await checkForUpdates();
            if (hasUpdate) {
                visible = true;
            }
        } catch (error) {
            console.error('Failed to check for updates:', error);
        }
    });

    async function handleUpdate() {
        isUpdating = true;
        updateError = '';
        
        try {
            await performUpdate();
            // The app will restart automatically after update
        } catch (error) {
            console.error('Update failed:', error);
            updateError = 'Update failed. Please try again later.';
            isUpdating = false;
        }
    }

    async function handleDecline() {
        // User declined update, exit the app
        await exitApp();
    }
</script>

{#if visible}
    <div class="update-overlay">
        <div class="update-dialog">
            <div class="update-header">
                <h2>🔄 Update Available</h2>
            </div>
            
            <div class="update-content">
                <p>A new version of the application is available.</p>
                <p>Would you like to update now?</p>
                
                {#if updateError}
                    <div class="error-message">
                        {updateError}
                    </div>
                {/if}
            </div>
            
            <div class="update-actions">
                <button 
                    class="update-btn decline" 
                    on:click={handleDecline} 
                    disabled={isUpdating}
                >
                    No, Exit App
                </button>
                <button 
                    class="update-btn confirm" 
                    on:click={handleUpdate} 
                    disabled={isUpdating}
                >
                    {isUpdating ? 'Updating...' : 'Yes, Update Now'}
                </button>
            </div>
        </div>
    </div>
{/if}

<style>
.update-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background-color: rgba(0, 0, 0, 0.8);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 9999;
    backdrop-filter: blur(5px);
}

.update-dialog {
    background: linear-gradient(135deg, rgba(255, 255, 255, 0.95), rgba(255, 255, 255, 0.85));
    border-radius: 16px;
    padding: 2rem;
    max-width: 450px;
    width: 90%;
    box-shadow: 0 20px 40px rgba(0, 0, 0, 0.3);
    border: 1px solid rgba(255, 255, 255, 0.2);
}

@media (prefers-color-scheme: dark) {
    .update-dialog {
        background: linear-gradient(135deg, rgba(30, 30, 30, 0.95), rgba(20, 20, 20, 0.85));
        border-color: rgba(255, 255, 255, 0.1);
        color: #f6f6f6;
    }
}

.update-header {
    text-align: center;
    margin-bottom: 1.5rem;
}

.update-header h2 {
    margin: 0;
    color: #0077ff;
    font-size: 1.5rem;
    font-weight: 600;
}

.update-content {
    text-align: center;
    margin-bottom: 2rem;
    line-height: 1.6;
}

.update-content p {
    margin: 0.5rem 0;
    font-size: 1rem;
}

.error-message {
    background-color: rgba(220, 53, 69, 0.1);
    border: 1px solid rgba(220, 53, 69, 0.3);
    color: #dc3545;
    padding: 0.75rem;
    border-radius: 8px;
    margin-top: 1rem;
    font-size: 0.875rem;
}

.update-actions {
    display: flex;
    gap: 1rem;
    justify-content: center;
}

.update-btn {
    padding: 0.75rem 1.5rem;
    border: none;
    border-radius: 8px;
    font-size: 1rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s ease;
    min-width: 120px;
}

.update-btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
}

.update-btn.confirm {
    background-color: #0077ff;
    color: white;
}

.update-btn.confirm:hover:not(:disabled) {
    background-color: #0056cc;
    transform: translateY(-1px);
}

.update-btn.decline {
    background-color: #6c757d;
    color: white;
}

.update-btn.decline:hover:not(:disabled) {
    background-color: #5a6268;
    transform: translateY(-1px);
}

.update-btn:active {
    transform: translateY(0);
}
</style>
