<script lang="ts">
    export let label: string;
    export let value: any;
    export let type = 'text';
    export let onChange = (value: any) => {};
</script>

<style>
    .stat-field {
        display: flex;
        align-items: center;
        justify-content: space-between;
        border-radius: 8px;
        background-color: #f0f0f0;
        padding: 8px 12px;
        margin: 6px 0;
        font-family: sans-serif;
    }

    .label {
        font-weight: 600;
        color: #333;
    }

    input {
        border: none;
        background: transparent;
        text-align: right;
        width: 100px;
        font-size: 1em;
        outline: none;
    }

    input:focus {
        background-color: #fff;
        border: 1px solid #ccc;
        border-radius: 4px;
        padding: 2px 4px;
    }
</style>

<div class="stat-field">
    <span class="label">{label}</span>
    <input bind:value={value} type={type} on:input={() => onChange(value)} />
</div>
