<script lang="ts">
    export let config: any;
    export let type: string;
</script>

<div class="stats-group">
    {#if type === 'SingleFire'}
        <p>Trigger Delay: {config.trigger_delay_ms} ms</p>
        <p>Recoil Completion: {config.recoil_completion_ms} ms</p>
        <p>Release Delay: {config.release_delay_ms} ms</p>
        <p>DX/DY: {config.dx}/{config.dy}</p>
        <p>Mag Size: {config.mag_size}</p>
    {:else if type === 'FullAutoStandard'}
        <p>RPM: {config.rpm}</p>
        <p>First Shot Scale: {config.first_shot_scale}</p>
        <p>Exponential Factor: {config.exponential_factor}</p>
        <p>DX/DY: {config.dx}/{config.dy}</p>
        <p>Mag Size: {config.mag_size}</p>
    {/if}
</div>

<style>
    .stats-group {
        margin-top: 0.5rem;
        padding-left: 1rem;
        font-size: 0.9em;
    }
</style>
