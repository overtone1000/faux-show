<script lang="ts">
	import IconSvg from "./icon_svg.svelte";

    type Props = {
        path:string,
        label:string,
        action:()=>void,
        disabled?:boolean
    };
    let { 
        path,
        label,
        action,
        disabled
    }:Props = $props();
</script>

<button
    class="iconbutton"
    aria-label={label}
    onclick={action}
    disabled={disabled}
>
    <IconSvg path={path}/>
    
</button>

<style>
    .iconbutton{
        border-radius: 5%;
        /*width:48px;*/
        height:100%;
        aspect-ratio: 1;
        align-self: center;
        margin:2px;
        padding:0px;
        margin: 0px;
        border-width: 2px;
        margin-right: var(--right_margin, "0px");
    }
    .iconsvg{
        fill:none;
        stroke:black;
        stroke-width:1;
    }
</style>