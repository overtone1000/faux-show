<script lang="ts">
	import { onDestroy, onMount } from "svelte";
    import "@fontsource/inter";
    
    let time=$state(new Date());

    let update_id:number|undefined=undefined;
    function update()
    {
        console.debug("Update.");
        time=new Date();
        update_id=setTimeout(update,60-time.getSeconds());
    }

    onMount(()=>{
        update();
    });

    onDestroy(()=>{
        clearTimeout(update_id);
    });

    const months=[
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December"
    ];

    function format_doubledigit(num:number)
    {
        let str=num.toString();
        if(str.length===2)
        {
            return str;
        }
        else
        {
            return "0"+str;
        }
    }
</script>

<div class="main">
    <div>{time.getHours()+":"+format_doubledigit(time.getMinutes())}</div>
    <div>{months[time.getMonth()] + " " + time.getDate() + ", " + time.getFullYear()}</div>
</div>

<style>
    .main
    {
        white-space: nowrap;
        font-family: Inter;
        font-size: xx-large;
        display:flex;
        flex-direction:row;
        justify-content: space-between;
        align-items: center;
        min-width: 400px;
        width: 400px;
        max-width: 400px;
    }
</style>