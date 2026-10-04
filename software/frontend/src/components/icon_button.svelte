<script lang="ts">
	import { derived } from "svelte/store";
	import IconSvg from "./icon_svg.svelte";

    type Props = {
        path:string,
        label:string,
        action:()=>void,
        disabled?:boolean,
        bgcolor?:string,
        color?:string
    };
    let { 
        path,
        label,
        action,
        disabled,
        color,
        bgcolor
    }:Props = $props();

    let dynamic_style=$derived.by(
        ()=>{
            let retval:string="";
            if(bgcolor)
            {
                retval="background-color:"+bgcolor;
            }
            return retval;
        }
    );

    console.debug($inspect(dynamic_style));

</script>

<button
    class="iconbutton"
    aria-label={label}
    onclick={action}
    disabled={disabled}
    style={dynamic_style}
>
    <IconSvg path={path} color={color}/>
    
</button>

<style>
    .iconbutton{
        border-radius: 5%;
        /*width:48px;*/
        height:100%;
        aspect-ratio: 1;
        align-self: center;
        margin:1px;
        padding:0px;
        border-width: 0px;
        /*margin-right: var(--right_margin, "0px");*/
    }
</style>