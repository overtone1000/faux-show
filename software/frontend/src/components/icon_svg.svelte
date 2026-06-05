<script lang="ts">
    type Props = {
        path:string,
        stroke?:string,
        stroke_width?:number
    };
    let { 
        path,
        stroke="black",
        stroke_width=1
    }:Props = $props();
</script>

<svg viewBox="0 0 24 24">
    <path 
        class="iconsvg" 
        style:stroke={stroke}
        style:stroke_width={stroke_width}
        d={path}
    />
</svg>

<style>
    .iconsvg{
        fill:none;
    }
</style>