import { format_time_remaining } from "./time";

export type Timer =
{
    //end:Date //Don't use date because it is hard to serialize.
    millis:number //This is either milliseconds left (if paused) or milliseconds in epoch when timer is finished
    paused:boolean
};

export type TimerState =
{
    timers:Timer[]
};

export function get_timer_end_millis(timer:Timer)
{
    if(timer.paused)
    {
        return Date.now()+timer.millis;
    }
    else
    {
        return timer.millis;
    }
}

export function get_timer_end_date(timer:Timer)
{
    return new Date(get_timer_end_millis(timer));
}

export function pause_timer(timer:Timer)
{
    if(!timer.paused)
    {
        timer.paused=true;
        timer.millis=timer.millis-Date.now();
    }
}

export function resume_timer(timer:Timer)
{
    if(timer.paused)
    {
        timer.paused=false;
        timer.millis=timer.millis+Date.now();
    }
}

export function timer_expired(timer:Timer)
{
    if(timer.paused)
    {
        return timer.millis<=0;
    }
    else
    {
        return Date.now()>=timer.millis;
    }
}