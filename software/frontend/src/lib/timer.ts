import { format_time_remaining } from "./time";
import type { TimerExtendedState } from "./timer_ext";

export type Timer =
{
    //end:Date //Don't use date because it is hard to serialize.
    name:string,
    millis:number, //This is either milliseconds left (if paused) or milliseconds in epoch when timer is finished
    paused:boolean
};

export function get_timer_end_millis(current_epoch_millis:number, timer:Timer)
{
    if(timer.paused)
    {
        return current_epoch_millis+timer.millis;
    }
    else
    {
        return timer.millis;
    }
}

export function get_timer_end_date(current_epoch_millis:number, timer:Timer)
{
    return new Date(get_timer_end_millis(current_epoch_millis, timer));
}

export function pause_timer(timer:Timer)
{
    if(!timer.paused)
    {
        timer.paused=true;
        timer.millis=timer.millis-Date.now(); //Set millis to the time remaining.
    }
}

export function resume_timer(timer:Timer)
{
    if(timer.paused)
    {
        timer.paused=false;
        timer.millis=timer.millis+Date.now(); //Set millis to the epoch millis when it will be done.
    }
}

export function timer_expired(current_epoch_millis:number, timer:Timer)
{
    if(timer.paused)
    {
        return timer.millis<=0;
    }
    else
    {
        return current_epoch_millis>=timer.millis;
    }
}