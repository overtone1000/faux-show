import { format_time_remaining } from "./time";
import { get_timer_end_date, timer_expired, type Timer } from "./timer";
import { timer_input_to_running_timer, type TimerInput } from "./timer_input";

export type TimerExtendedState={
    expired:boolean,
    end_date:Date,
    formatted_remaining_time:string,
    display_class:string
};

export type FullTimer={timer:Timer, state:TimerExtendedState};

export type AllTimers = FullTimer[];

//

function get_extended_state(now:number, nowdate:Date, timer:Timer):TimerExtendedState
{
    const expired = timer_expired(now,timer);
    const end_date = get_timer_end_date(now,timer);

    let formatted_remaining_time:string;
    let display_class:string;
    if(expired)
    {
        formatted_remaining_time="00:00:00";
        display_class="timer_entry expired";
    }
    else
    {
        formatted_remaining_time = format_time_remaining(nowdate,end_date);
        display_class="timer_entry";
    }

    return {
        expired,
        end_date,
        formatted_remaining_time,
        display_class
    };
}

export function update_extended_states(nowdate:Date, timers:AllTimers)
{
    //const nowdate=new Date();
    const now=nowdate.valueOf();
    for(let full_timer of timers)
    {
        full_timer.state=get_extended_state(now,nowdate,full_timer.timer);
    }
}

export function new_timer(name:string, input:TimerInput, timers:AllTimers)
{
    const timer:Timer = timer_input_to_running_timer(name,input);
    const nowdate=new Date();
    const now=nowdate.valueOf();
    timers.push({
        timer,
        state:get_extended_state(now,nowdate,timer)
    });
}

export function all_timers_to_serial(timers:AllTimers)
{
    let timer_array=[];
    for(let full_timer of timers)
    {
        timer_array.push(full_timer.timer);
    }
    return JSON.stringify(timer_array);
}

export function all_timers_from_serial(json:string):AllTimers
{
    let timer_array:Timer[]=JSON.parse(json);
    let retval=[];
    const nowdate=new Date();
    const now=nowdate.valueOf();
    for(const timer of timer_array)
    {
        retval.push(
            {
                timer,
                state:get_extended_state(now,nowdate,timer)
            }
        );
    }
    return retval;
}