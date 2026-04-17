<script lang="ts">
	import { mdiCheck, mdiDelete, mdiPlus } from "@mdi/js";
	import IconButton from "./icon_button.svelte";
	import type { ModifyTimerState } from "./time_input.svelte";
	import { get_empty_timer, type NewTimerState, new_timer_to_end } from "$lib/timer";
	import TimeInput from "./time_input.svelte";
	import { onDestroy, onMount } from "svelte";
	import { format_time_remaining } from "$lib/time";
    
    export type Timer =
    {
        end:Date
    };

    export type TimerState =
    {
        timers:Timer[]
    };

    let timer_state:TimerState = $props();

    let time=$state(new Date());
    let update_id:number|undefined=undefined;
    function update()
    {
        time=new Date();
        update_id=setTimeout(update,100);
    }

    onMount(()=>{
        update();
    });

    onDestroy(()=>{
        clearTimeout(update_id);
    });

    function add_timer()
    {
        new_timer=get_empty_timer();
        show_add_display=true;
    }

    function save_timer()
    {
        const end = new_timer_to_end(new_timer);
        
        timer_state.timers.push(
            {
                end
            }
        );

        show_add_display=false
    }       
    

    let show_add_display:boolean=$state(true);
    let new_timer:NewTimerState=$state(get_empty_timer());
</script>

<div class="main">
    {#if show_add_display}
        <div class="timer_edit_container">
            <div class="timer_edit">
                <TimeInput new_timer={new_timer}/>
            </div>
            <div class="bottom_row">
                <div class="icon_container">
                    <IconButton
                        path={mdiDelete}
                        label={"discard"}
                        action={()=>{show_add_display=false}} 
                    />
                </div>
                <div class="icon_container">
                    <IconButton
                        path={mdiCheck}
                        label={"save"}
                        action={save_timer} 
                    />
                </div>
            </div>
        </div>
    {:else}
        <div class="spacer"></div>
        {#each timer_state.timers as timer}
            <div>
                {format_time_remaining(time,timer.end)}
            </div>
            <div class="spacer"></div>
        {/each}
        <div class="icon_container">
            <IconButton 
                path={mdiPlus}
                label={"add timer"}
                action={add_timer} 
            />
        </div>
    {/if}
</div>

<style>
    .main
    {
        display: flex;
        flex-direction: column;
        height: 100%;
        width: 100%;
        justify-content: space-between;
        align-items: center;
    }
    .icon_container
    {
        display: flex;
        flex-direction: column;
        height: 16mm;
    }
    .spacer
    {
        flex-grow: 1;
    }
    .timer_edit_container
    {
        width:100%;
        height:100%;
        display: flex;
        flex-direction: column;
    }
    .timer_edit
    {
        flex-grow:1;
    }
    .bottom_row
    {
        display:flex;
        flex-direction: row;
        justify-content: space-between;
    }
</style>