<script lang="ts">
	import type { TimerInput } from "$lib/timer_input";
	import SingleDigitInput from "./single_digit_input.svelte";

    export type ModifyTimerState =
    {
        new_timer_input:TimerInput
    };

    let {
        new_timer_input: new_timer=$bindable()
    }:ModifyTimerState = $props();
</script>

<div class="main">
    <div class="pair">
        <SingleDigitInput bind:value={new_timer.hours_tens} min={0} max={9}/>
        <SingleDigitInput bind:value={new_timer.hours_ones} min={0} max={9}/>
    </div>
    <div class="colon">:</div>
    <div class="pair">
        <SingleDigitInput bind:value={new_timer.minutes_tens} min={0} max={5}/>
        <SingleDigitInput bind:value={new_timer.minutes_ones} min={0} max={9}/>
    </div>
    <div class="colon">:</div>
    <div class="pair">
        <SingleDigitInput bind:value={new_timer.seconds_tens} min={0} max={5}/>
        <SingleDigitInput bind:value={new_timer.seconds_ones} min={0} max={9}/>
    </div>
</div>

<style>
    .main
    {
        display: flex;
        flex-direction: row;
        height: 100%;
        width: 100%;
        justify-content: center;
        align-items: center;
    }
    .colon
    {
        font-size: 20vh;
    }
    .pair
    {
        height: 100%;
        width: 25vw;
        display: flex;
        flex-direction: row;
    }
</style>