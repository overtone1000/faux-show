<script lang="ts">
	import { mdiCheck, mdiDelete, mdiPlus, mdiPause, mdiPlay } from "@mdi/js";
	import IconButton from "./icon_button.svelte";
	import TimeInput from "./time_input.svelte";
	import { format_time_remaining } from "$lib/time";
	import { get_timer_end_date, pause_timer, resume_timer, type Timer, type TimerState } from "$lib/timer";
	import { get_empty_timer, timer_input_to_running_timer, type TimerInput } from "$lib/timer_input";

    type Props =
    {
        time:Date,
        timer_state:TimerState|undefined
    };
    let {time, timer_state=$bindable()}:Props = $props();


    function add_timer()
    {
        console.debug("Adding timer.");
        new_timer_input=get_empty_timer();
        show_add_display=true;
    }

    function save_timer()
    {
        console.debug("Saving timer.");
        const new_running_timer = timer_input_to_running_timer(new_timer_input);
        
        if(timer_state)
        {
            timer_state.timers.push(new_running_timer);
        }

        show_add_display=false
    }       
    

    let show_add_display:boolean=$state(timer_state===undefined || timer_state.timers.length==0);
    let new_timer_input:TimerInput=$state(get_empty_timer());
</script>

<div class="main">
    {#if show_add_display}
        <div class="timer_edit_container">
            <div class="timer_edit">
                <TimeInput bind:new_timer_input={new_timer_input}/>
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
        <div class="table_container">
            {#if timer_state}
                <table class="timertable">
                    <thead>
                        
                    </thead>
                    <tbody>
                    {#each timer_state.timers as timer, index}
                        <tr class="timer_entry">
                            <td>{format_time_remaining(time,get_timer_end_date(timer))}</td>
                            {#if timer.paused}
                                <td>
                                    <div class="icon_container">
                                        <IconButton
                                            path={mdiPlay}
                                            label={"resume_timer"}
                                            action={()=>{resume_timer(timer);}} 
                                        />
                                    </div>
                                </td>
                            {:else}
                                <td>
                                    <div class="icon_container">
                                        <IconButton
                                            path={mdiPause}
                                            label={"pause_timer"}
                                            action={()=>{pause_timer(timer);}} 
                                        />
                                    </div>
                                </td>
                            {/if}
                            <td>
                                <div class="icon_container">
                                    <IconButton
                                        path={mdiDelete}
                                        label={"discard_timer"}
                                        action={()=>{timer_state.timers.splice(index,1)}} 
                                    />
                                </div>
                            </td>
                        </tr>
                    {/each}
                    </tbody>
                </table>
            {/if}
            <div class="spacer"></div>
        </div>
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
    .table_container
    {
        width: 100%;
        height: 100%;
        overflow: scroll;
    }
    .timertable
    {
        margin: 0;
        width:100%;
        flex-shrink: 1;
    }
    .timer_entry
    {
        height: 10px;
        font-size: 50px;
    }
    td {
        border-left: none;
        border-right: none;
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