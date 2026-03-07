<script lang="ts">
    type Props = {
        path:string,
        label:string,
        action:()=>void
    };
    let { 
        path,
        label,
        action
    }:Props = $props();
</script>

<button
    class="iconbutton"
    aria-label={label}
    onclick={action}
>
    <svg viewBox="0 0 24 24">
        <path class="iconsvg" d={path}/>
    </svg>
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
    }
    .iconsvg{
        fill:none;
        stroke:black;
        stroke-width:1;
    }
</style>