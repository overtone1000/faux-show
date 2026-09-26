export type TimerInput =
{
    hours_tens:number,
    hours_ones:number,
    minutes_tens:number,
    minutes_ones:number,
    seconds_tens:number,
    seconds_ones:number
};

export const get_empty_timer = ()=>{
    const retval:TimerInput={
        hours_tens:0,
        hours_ones:0,
        minutes_tens:0,
        minutes_ones:0,
        seconds_tens:0,
        seconds_ones:0
    };
    return retval;
};

export function timer_input_to_running_timer(new_timer:TimerInput)
{
    const hours = new_timer.hours_tens*10 + new_timer.hours_ones;
    const minutes = new_timer.minutes_tens*10+new_timer.minutes_ones;
    const seconds = new_timer.seconds_tens*10+new_timer.seconds_ones;
    const total_millis = ((hours*60+minutes)*60+seconds)*1000;

    return {
        millis:Date.now()+total_millis,
        paused:false
    };
}
