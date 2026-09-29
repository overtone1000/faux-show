<script lang="ts">
	import { mdiCheck, mdiDelete, mdiPlus, mdiPause, mdiPlay } from "@mdi/js";
	import IconButton from "./icon_button.svelte";
	import TimeInput from "./time_input.svelte";
	import { format_time_remaining } from "$lib/time";
	import { get_timer_end_date, pause_timer, resume_timer, timer_expired, type Timer, type TimerState } from "$lib/timer";
	import { get_empty_timer, timer_input_to_running_timer, type TimerInput } from "$lib/timer_input";

    type Props =
    {
        time:Date,
        timer_state:TimerState|undefined
    };

    let {time, timer_state=$bindable()}:Props = $props();

    type TimerExtendedState={
        expired:boolean,
        end_date:Date,
        formatted_remaining_time:string,
        display_class:string
    };

    let timer_extended_states:TimerExtendedState[]=$derived.by(
        ()=>{
            if(timer_state)
            {
                return timer_state.timers.map(
                    (timer,index)=>{

                        const expired = timer_expired(timer);
                        const end_date = get_timer_end_date(timer);

                        let formatted_remaining_time:string;
                        let display_class:string;
                        if(expired)
                        {
                            formatted_remaining_time="00:00:00";
                            display_class="timer_entry expired";
                        }
                        else
                        {
                            formatted_remaining_time = format_time_remaining(time,end_date);
                            display_class="timer_entry";
                        }
                        
                        return {
                            expired,
                            end_date,
                            formatted_remaining_time,
                            display_class
                        };
                    }
                );
            }
            else
            {
                return [];
            }
        }
    );


    function add_timer()
    {
        console.debug("Adding timer.");
        new_timer_input=get_empty_timer();
        show_add_display=true;
    }

    function save_timer()
    {
        console.debug("Saving timer.");
        let number_of_timers:number;
        if(timer_state)
        {   
            number_of_timers=timer_state.timers.length;
        }
        else
        {
            number_of_timers=0;   
        }
        const name = "Timer " + (number_of_timers+1);
        const new_running_timer = timer_input_to_running_timer(name, new_timer_input);
        
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
                        <tr class={timer_extended_states[index].display_class}>
                            <td>
                                {timer.name}
                            </td>
                            <td>
                                {timer_extended_states[index].formatted_remaining_time}
                            </td>
                            <td>
                            {#if !timer_extended_states[index].expired}
                                {#if timer.paused}
                                    <div class="icon_container">
                                        <IconButton
                                            path={mdiPlay}
                                            label={"resume_timer"}
                                            action={()=>{resume_timer(timer);}} 
                                        />
                                    </div>
                                {:else}
                                    <div class="icon_container">
                                        <IconButton
                                            path={mdiPause}
                                            label={"pause_timer"}
                                            action={()=>{pause_timer(timer);}} 
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