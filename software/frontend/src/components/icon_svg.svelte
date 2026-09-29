<script lang="ts">
    type Props = {
        path:string,
        color?:string,
    };
    let { 
        path,
        color="black",
    }:Props = $props();
</script>

<svg viewBox="0 0 24 24">
    <path 
        class="iconsvg" 
        style:fill={color}
        d={path}
    />
</svg>

<style>
    .iconsvg{
        stroke-width:0px;
    }
</style>