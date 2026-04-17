<script lang="ts">
	import { mdiArrowDown, mdiArrowUp, mdiCheck, mdiDelete, mdiPlus } from "@mdi/js";
	import IconButton from "./icon_button.svelte";

    export type State =
    {
        value:number,
        min:number,
        max:number
    };

    let {value=$bindable(), min, max}:State = $props();

    function changeval(change:number)
    {
        if(value+change<=max && value+change>=min)
        {
            value+=change;
        }
    }
</script>

<div class="main">
    <div class="arrow">
        <IconButton 
            path={mdiArrowUp}
            label={"increment"}
            action={()=>{changeval(+1)}}
            disabled={value>=max}
        />
    </div>
    <div class="value">
        {value}
    </div>
    <div class="arrow">
        <IconButton 
            path={mdiArrowDown}
            label={"decrement"}
            action={()=>{changeval(-1)}}
            disabled={value<=min}
        />
    </div>
</div>

<style>
    .main
    {
        display: flex;
        flex-direction: column;
        height: 100%;
        width: 100%;
        justify-content: center;
        align-items: center;
    }
    .value
    {
        height: 40%;
        align-content: center;
        text-align: center;
        font-size: 8em;
    }
    .arrow
    {
        height: 20%
    }
</style>