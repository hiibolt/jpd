<script lang="ts">
    import { config } from "../stores/state";

    export let label: string;
    export let value: any;
    export let type = 'text';
    export let description: string | null = null;
    export let onChange = (value: any) => {};

    // If the label contains "(ms)", treat it as a number input
    const isFloat = label.includes("DX") || 
        label.includes("DY") ||
        label.includes("Multiplier");
    
    function handleCharKeybindInput(event: KeyboardEvent) {
        const char = event.key;
        if (/^[0-9a-zA-Z]$/.test(char)) {
            onChange(char);

            value = char;
        }
    }
</script>

<style>
    .label-container {
        display: flex;
        flex-direction: column;
        gap: 4px;
        text-align: left;
    }
    .stat-field {
        display: flex;
        align-items: center;
        justify-content: space-between;
        border-radius: 8px;
        border: 1px solid var(--border-color, #ccc);
        background-color: var(--card-bg);
        padding: 8px 12px;
        margin: 6px 0;
        font-family: sans-serif;
    }

    .label {
        font-weight: 600;
        color: #FFFFFF;
    }

    input {
        border: none;
        background: transparent;
        text-align: right;
        width: 100px;
        font-size: 1em;
        color: #FFFFFF;
    }
    input:focus {
        background-color: var(--card-bg);
    }
</style>

<div class="stat-field">
    <div class="label-container">
        <span class="label">{label}</span>
        {#if description}
            <span class="description">{description}</span>
        {/if}
    </div>

    {#if type === 'char'}
        <input
            step={isFloat ? 0.05 : 1}
            bind:value={value}
            type='text'
            maxlength="1"
            on:keydown|preventDefault={handleCharKeybindInput}
        />
    {:else}
        <input
            step={isFloat ? 0.05 : 1}
            bind:value={value}
            type={type}
            on:input={() => onChange(value)}
        />
    {/if}
</div>
