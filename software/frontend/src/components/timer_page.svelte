<script lang="ts">
	import { mdiCheck, mdiDelete, mdiPlus, mdiPause, mdiPlay } from "@mdi/js";
	import IconButton from "./icon_button.svelte";
	import TimeInput from "./time_input.svelte";
	import { get_empty_timer, timer_input_to_running_timer, type TimerInput } from "$lib/timer_input";
	import { pause_timer, resume_timer } from "$lib/timer";
	import { new_timer, type AllTimers } from "$lib/timer_ext";

    type Props =
    {
        timers:AllTimers,
        save_timers:()=>void
    };

    let {timers=$bindable(),save_timers}:Props = $props();

    function add_timer()
    {
        console.debug("Adding timer.");
        new_timer_input=get_empty_timer();
        show_add_display=true;
    }

    function save_timer()
    {
        console.debug("Saving timer.");
        const name = "Timer " + (timers.length+1);
        new_timer(name,new_timer_input,timers);
        save_timers();
        show_add_display=false;
    } 
    

    let show_add_display:boolean=$state(timers.length<=0);
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
            <table class="timertable">
                <thead>
                </thead>
                <tbody>
                {#each timers as full_timer, index}
                    <tr class={full_timer.state.display_class}>
                        <td>
                            {full_timer.timer.name}
                        </td>
                        <td>
                            {full_timer.state.formatted_remaining_time}
                        </td>
                        <td>
                        {#if !full_timer.state.expired}
                            {#if full_timer.timer.paused}
                                <div class="icon_container">
                                    <IconButton
                                        path={mdiPlay}
                                        label={"resume_timer"}
                                        action={()=>{resume_timer(full_timer.timer);}} 
                                    />
                                </div>
                            {:else}
                                <div class="icon_container">
                                    <IconButton
                                        path={mdiPause}
                                        label={"pause_timer"}
                                        action={()=>{pause_timer(full_timer.timer);}} 
                                    />
                                </div>
                            {/if}
                        {/if}
                        </td>
                        <td>
                            <div class="icon_container">
                                <IconButton
                                    path={mdiDelete}
                                    label={"discard_timer"}
                                    action={()=>{timers.splice(index,1); save_timers();}} 
                                />
                            </div>
                        </td>
                    </tr>
                {/each}
                </tbody>
            </table>
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
        height: 40mm;
        font-size: 15mm;
    }
    .timer_entry.expired
    {
        background-color: rgb(77, 2, 2);
        animation: bg_oscillation 1s infinite alternate ease-in-out
    }
    @keyframes bg_oscillation{
        0% {
            background-color:  rgb(77, 2, 2);
        }
        100% {
            background-color: #3f2003;
        } 
    }
    td {
        border-left: none;
        border-right: none;
    }
    .icon_container
    {
        display: flex;
        flex-direction: column;
        height: 20mm;
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