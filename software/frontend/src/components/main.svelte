<script lang="ts">
    import { mdiCheck, mdiCircleOffOutline, mdiCircleOutline, mdiClock, mdiCross, mdiDebugStepInto, mdiImageMultiple, mdiEarHearing } from '@mdi/js';
    import { mdiRefresh } from '@mdi/js';
    import { mdiRobot } from '@mdi/js';
    import IconTab, { type TabProps } from './icon_tab.svelte';
	import { onDestroy, onMount } from 'svelte';
	import Time from './time.svelte';
	import { VoiceControlState, type AutoTabEntry, type Command, type AutoTab, type VoiceControlOptions } from '$lib/commands';
	import Slideshow from './slideshow.svelte';
	import IconSvg from './icon_svg.svelte';
	import TimerPage from './timer_page.svelte';
	import { all_timers_from_serial, all_timers_to_serial, update_extended_states, type AllTimers } from '$lib/timer_ext';
	import IconButton from './icon_button.svelte';
	  
    //Hook console;
    enum ConsoleType {
        Debug,
        Error
    }

    type ConsoleEntry = {time:number, type:ConsoleType, args:any[]};
    const console_history:ConsoleEntry[] = $state([]);
    const baseline_debug_function=console.debug;
    const baseline_error_function=console.error;
    function trim_console_history()
    {
        const DESIRED_SIZE=50;
        if(console_history.length>DESIRED_SIZE)
        {
            console_history.splice(0,console_history.length-DESIRED_SIZE);
        }
    }
    console.debug = function (...args:any[]){
        console_history.push({
            time:Date.now(),
            type:ConsoleType.Debug,
            args:args
        });
        trim_console_history();
        baseline_debug_function.apply(console,args);
    };
    console.error = function (...args:any[]){
        console_history.push({
            time:Date.now(),
            type:ConsoleType.Error,
            args:args
        });
        trim_console_history();
        baseline_error_function.apply(console,args);
    };

    console.debug("Starting main. Debug v1.");

    enum MainField {
        iframe,
        component
    };

    type IFrameMeta = {
        url:string|null,
        title:string
    };

    enum ComponentType {
        clock,
        slideshow
    };

    type Main = {
        field:MainField,
        iframe_meta?:IFrameMeta
        component_meta?:ComponentType
    };


    type TabsConfig = {
        label:string,
        title:string,
        icon_path:string,
        url:string
    };

    let main:Main|undefined = $state(undefined);
    let display_on:boolean = $state(true);

    //const timers:Timer[] = $state([]);

    let tabs:TabProps[]|undefined = $state(undefined);
    
    const clock:TabProps = {
        action: () => {
            main={
                field: MainField.component,
                component_meta:ComponentType.clock
            }
        },
        icon_label: "clock",
        icon_path: mdiClock,
        disabled: false
    };

    const slideshow:TabProps = {
        action: () => {
            main={
                field: MainField.component,
                component_meta:ComponentType.slideshow
            }
        },
        icon_label: "slideshow",
        icon_path: mdiImageMultiple,
        disabled: false
    };

    let show_debug:boolean=$state(false);
    let show_voice_instructions:boolean=$state(false);

    function set_manual_tab(tab_props:TabProps)
    {
        manual_tab_props=tab_props;
        active_tab_props=tab_props;
        //tab_props.action(); //Don't need to do this, it will run in the effect below.
    }

    let manual_tab_props=$state<TabProps|null>(null);
    let auto_tab_config=$state<AutoTab|null>(null);

    let active_tab_props=$state<TabProps|null>(null);

    $effect(()=>{
        active_tab_props?.action();
    });
        
    let auto_tabs:Set<AutoTabEntry>=new Set();

    let auto_tab_props = $derived(
        {
            action: () => {
                if(auto_tab_config!==null)
                {
                    main={
                        field: MainField.iframe,
                        iframe_meta:{
                            url: auto_tab_config.url,
                            title: "Automatic Tab"
                        }
                    }
                }
            },
            icon_label: "auto",
            icon_path: mdiRobot,
            disabled: auto_tab_config===null
        }
    );

    function update_auto_tab_state(update_active_tab:boolean)
    {
        console.debug("Updating auto tab state.",update_active_tab);

        let selected_config:undefined|AutoTabEntry=undefined;
        const now=new Date();
        for(const auto_tab_entry of auto_tabs)
        {
            if(auto_tab_entry.expiry<now)
            {
                auto_tabs.delete(auto_tab_entry);
            }
            else
            {
                if(
                    selected_config===undefined || 
                    auto_tab_entry.config.priority>selected_config.config.priority ||
                    (
                        auto_tab_entry.config.priority==selected_config.config.priority &&
                        selected_config.expiry<auto_tab_entry.expiry
                    )
                )
                {
                    console.debug(selected_config,auto_tab_entry,"This entry beats current entry. Updating selected config.");
                    selected_config=auto_tab_entry;
                }
                else
                {
                    console.debug(selected_config,auto_tab_entry,"Current entry wins. Keeping current entry.");
                }
            }
        }

        console.debug("Selected state is",selected_config);

        if(selected_config!==undefined)
        {
            if(selected_config.config!==auto_tab_config)
            {
                auto_tab_config=selected_config.config;
            }
            if(update_active_tab)
            {
                active_tab_props=auto_tab_props;
            }
            let wait=(selected_config.expiry.getTime()-now.getTime());
            console.debug("Setting timeout for update.",wait);
            setTimeout(()=>{update_auto_tab_state(false)},wait);
        }
        else
        {
            auto_tab_config=null;
            if(active_tab_props!==manual_tab_props)
            {
                active_tab_props=manual_tab_props;
            }
        }
    }

    let photoprism_key=$state<string|undefined>(undefined);
    let has_key=$state<string|undefined>(undefined);
    let voice_control_state=$state<VoiceControlState>(VoiceControlState.NotEnabled);
    let voice_control_options=$state<VoiceControlOptions>([]);
    function handle_server_command(command:Command)
    {
        console.debug("Handling command.");
        if(command.AutoTab)
        {
            let auto_tab:AutoTab=command.AutoTab;
            console.debug("Received auto tab.",auto_tab);

            if(auto_tab && auto_tab.url && auto_tab.priority && auto_tab.timeout_seconds)
            {
                let expiry:Date = new Date(Date.now()+auto_tab.timeout_seconds*1000);

                const entry:AutoTabEntry = {
                    config:auto_tab,
                    expiry:expiry
                };

                auto_tabs.add(entry);
                update_auto_tab_state(true);
            }
            else
            {
                console.error("Malformed auto tab.",auto_tab);
            }
        }
        
        if(command.PhotoprismKey)
        {
            console.debug("Received photoprism key.");
            photoprism_key=command.PhotoprismKey;
        }

        if(command.HASKey)
        {
            console.debug("HAS key received.");
            has_key=command.HASKey;
            //speak("Hey, TTS is working!");
        }

        if(command.VoiceControlOptions)
        {
            console.debug("Voice control options received.");
            voice_control_options=command.VoiceControlOptions;
        }
        
        if(command.SetScreenState!==undefined)
        {
            display_on=command.SetScreenState
        }

        if(command.SetVoiceControlState)
        {
            voice_control_state=command.SetVoiceControlState as VoiceControlState;
        }

        if(command.AcknowledgeAlarms)
        {
            let timers_to_preserve=[];
            for(const timer of timers)
            {
                if(!timer.state.expired)
                {
                    timers_to_preserve.push(timer);
                }
            }
            timers=timers_to_preserve;
            save_timers();
        }
    }

    let socket:WebSocket|undefined=undefined;
    let socket_state:boolean=$state(false);
    function open_socket(){
        if(socket_url)
        {
            console.debug("Opening websocket");
            socket = new WebSocket(socket_url);

            // Connection opened
            socket.onopen=(event) => {
                console.debug("Connection opened.");
                socket_state=true;
            };

            // Listen for messages
            socket.onmessage = (event) => {
                console.log("Message from server ", event.data);
                handle_server_command(JSON.parse(event.data));
            };

            // Handle disconnect
            socket.onclose = (event)=>{
                setTimeout(open_socket,1000);
                socket_state=false;
            };
        }
    };

    function build_tabs(tabs_config:TabsConfig[]) {
        if(tabs_config.length>0)
        {
            tabs = [];

            for(const tabconfig of tabs_config)
            {
                tabs.push(
                    {
                        icon_label: tabconfig.label,
                        icon_path: tabconfig.icon_path,
                        action: ()=>{
                            main={
                                field: MainField.iframe,
                                iframe_meta:{
                                    url: tabconfig.url,
                                    title: tabconfig.title
                                }
                            }
                        }
                    }
                );
            }

            //Default to zero
            set_manual_tab(tabs[0]);
        }
    }

    async function get_tabs() {
        const url = location.origin+"/config/tabs.json";
        console.debug("Getting tabs from " + url);
        try {
            const response = await fetch(url);
            if (!response.ok) {
                throw new Error(`Response status: ${response.status}`);
            }
            else
            {
                const result = await response.json();
                build_tabs(result);
            }
        } catch (error:any) {
            console.error("Tab retrieval failed.",error.message);
        }
    }

    // TTS
    function speak(text:string)
    {
        console.warn("Need to make this and some of the JSON below configurable via environment variables.");
        //For pico tts
        //const BASE="http://10.10.10.10:8123/api/services/tts/speak";
        //For cloud say
        const BASE="http://10.10.10.10:8123/api/services/tts/cloud_say";

        if(has_key)
        {
            fetch(
                BASE,
                {
                    method: "POST",
                    //mode: "no-cors",
                    headers: {
                        "Content-Type": "application/json",
                        "Authorization": "Bearer " + has_key,
                        //"Origin": window.location.origin
                    },
                    //For picoTTS
                    /*
                    body: JSON.stringify(
                        {
                            entity_id:"tts.pico_tts_en_us",
                            media_player_entity_id:"media_player.kitchen_2_2",
                            message:text
                        }
                    )
                    */
                    //For cloud say
                    body: JSON.stringify(
                        {
                            entity_id:"media_player.kitchen_2_2",
                            message:text
                        }
                    )
                }
            ).then(
                ((resp)=>{
                    console.debug("Speak result:",resp);
                })
            )
        }
    }

    // Timer State
    const timer_storage_key="timers";
    let time=$state(new Date());
    let update_time_id:number|undefined=undefined;
    function update_time()
    {
        time=new Date();
        update_time_id=setTimeout(update_time,1000);
    }

    let update_announcement_id:number|undefined=undefined;
    function timer_finished_announcement()
    {
        console.debug("Callback running.");

        let expired_names:string[] = [];
        for(const full_timer of timers)
        {
            if(full_timer.state.expired)                
            {
                expired_names.push(full_timer.timer.name)
            }
        }

        console.debug("Expired names",expired_names);

        if(expired_names.length>0)
        {
            if(expired_names.length>1)
            {
                let speech="";
                for(let n=0;n<expired_names.length-1;n++)
                {
                    speech=speech+expired_names[n] + ", ";
                }
                speech=speech+ " and " + expired_names[expired_names.length-1] + " are done.";
                speak(speech);
            }
            else
            {
                speak(expired_names[0] + " is done.");
            }
        
            update_announcement_id=setTimeout(timer_finished_announcement, 10000);
        }
        else
        {
            clearTimeout(update_announcement_id);
        }
    }

    let timers:AllTimers = $state([]); //need to start undefined for initialization from localStorage in effect below
    let timers_initialized:boolean=false;

    function timer_onMount(){
        let timer_state_json = localStorage.getItem(timer_storage_key);
        if(timer_state_json)
        {
            console.debug("Loading timers from storage.",timer_state_json);
            timers=all_timers_from_serial(timer_state_json);
            timers_initialized=true;
        }
        update_time();
    }

    function timer_onDestroy(){
        clearTimeout(update_time_id);
        clearTimeout(update_announcement_id);
    }

    $effect(
        ()=>{
            console.info("Updating extended states.");
            update_extended_states(time, timers);
            //setTimeout(timer_finished_announcement);
        }
    );

    $effect(
        ()=>{
            //If update announcement isn't running, check if any timers are expired and restart.
            if(!update_announcement_id)
            {
                for(const full_timer of timers)
                {
                    if(full_timer.state.expired)                
                    {
                        console.info("Expired timer found, calling callback.");
                        setTimeout(timer_finished_announcement);
                        break;
                    }
                }   
            }
        }
    )

    const save_timers = () =>
    {
        if(timers_initialized)
        {
            //CANNOT USE DEBUG OR ERROR in effects because of the overload, causes this effect to run repeatedly!!
            const serial=all_timers_to_serial(timers);
            console.info("Saving timers to storage.",serial)
            localStorage.setItem(timer_storage_key,serial);
        }
        else
        {
            console.info("Skipping timer save because not mounted.");
        }
    }

    $effect(
        ()=>{
            //Save timers when changed at this level (could happen by voice command)
            save_timers()
        }
    );

    //

    let socket_url:undefined|string = undefined;
    onMount(()=>{
        //development mode flag
        socket_url = import.meta.env.DEV ? "ws:/127.0.0.1:30125" : "ws:/"+location.host;
        
        open_socket();
        get_tabs();
        timer_onMount();
    });

    onDestroy(()=>{
        timer_onDestroy();
    });

    let voice_control_icon_color=$derived.by(
        ()=>{
            switch(voice_control_state)
            {
                case VoiceControlState.NotEnabled:return "gray";
                case VoiceControlState.ListeningForWakeword:return "yellow";
                case VoiceControlState.StreamingToWhisper:return "green";
                default:return "black";
            }
        }
    );
</script>

<div class="whole_display">
    <div class="tab-row">
        {#each tabs as tab}
            <IconTab --right_margin="4px" props={tab}/>
        {/each}
        <IconTab props={clock}/>
        <IconTab props={slideshow}/>
        <IconTab props={auto_tab_props}/>
        <div class="spacer"></div>
        <Time/>
        <div class="spacer"></div>
        {#if socket_state}
            <div class="infotab">
                <IconButton path={mdiEarHearing} color={voice_control_icon_color} bgcolor="transparent" label="voice_control" action={()=>{show_voice_instructions=!show_voice_instructions}}/>
            </div>
            <div class="infotab">
                <IconSvg path={mdiCircleOutline} color="green"/>
            </div>
        {:else}
            <div class="infotab">
                <IconSvg path={mdiCircleOffOutline} color="red"/>
            </div>
        {/if}
        <IconButton path={mdiDebugStepInto} label="debug" action={()=>{show_debug=!show_debug;}}/>
        <IconButton path={mdiRefresh} label="refresh" action={()=>{location.reload();}}/>
    </div>
    <div class="main_outer">
        {#if display_on && main !== undefined}
            <div class="main">
                {#if main.field === MainField.iframe && main.iframe_meta !== undefined}
                    <iframe class="full-size" src={main.iframe_meta.url} title={main.iframe_meta.title}>
                        <p>iframe unsupported</p>
                    </iframe>
                {:else if main.field === MainField.component && main.component_meta !== undefined}
                    {#if main.component_meta === ComponentType.clock}
                        <TimerPage bind:timers save_timers={save_timers}/>
                    {:else if main.component_meta === ComponentType.slideshow}
                        <Slideshow photoprism_key={photoprism_key}/>
                    {/if}
                {/if}
            </div>
        {/if}
        {#if show_debug}
            <div class="overlay console">
                {#each console_history as entry}
                    {#if entry.type===ConsoleType.Debug}
                    <div class="console_entry">
                        {entry.time.toString() + ": " + entry.args.toString()}
                    </div>
                    {:else if entry.type===ConsoleType.Error}
                    <div class="console_entry error">
                        {entry.time.toString() + ": " + entry.args.toString()}
                    </div>
                    {/if}
                {/each}
            </div>
        {:else if show_voice_instructions}
            <div class="overlay voice_instructions">
                {#each voice_control_options as voice_option}
                    Need to explain option!
                {/each}
            </div>
        {/if}
    </div>
</div>

<style>
    .full-size
    {
        width:100%;
        height:100%;
        min-width:0%;
        min-height:0%;
        flex-grow: 1;
        flex-shrink: 1;
        overflow-y: hidden;
    }
    .whole_display
    {
        width: 100vw;
        height:100vh;
        margin: 0px;
        display:flex;
        flex-direction: column;
    }
    .tab-row
    {
        width: 100%;
        /*Make height absolute for touch device*/
        height: 16mm;
        display:flex;
        flex-direction: row;
    }
    .spacer
    {
        flex-shrink: true;
        width:100%;
    }
    .main_outer
    {
        width: 100%;
        height: 100%;
        overflow-y: hidden;
        overflow-x: hidden;
        display: grid;
        grid-template-columns: 100%;
        grid-template-rows: 100%;
        place-items: center;
    }
    .main
    {
        grid-area: 1 / 1;
        max-width:100%;
        max-height:100%;
        height:100%;
        width:100%;
    }
    .overlay
    {
        grid-area: 1 / 1;
        max-width:100%;
        max-height:100%;
        height:100%;
        width:100%;
    }
    .console
    {
        opacity: 0.75;
        background-color: black;
        color:white;
        display:flex;
        flex-direction: column;
        justify-content: end;
        font-size: small;
    }
    .console_entry
    {
        width:100%;
        height:min-content;
    }
    .voice_instructions
    {
        opacity: 0.75;
        background-color: #0d0150;
        color:white;
        display:flex;
        flex-direction: column;
        justify-content: end;
    }
    .error
    {
        color:red;
    }
    * {
        color-scheme: dark;
    }
    .infotab{
        border-radius: 5%;
        /*width:48px;*/
        height:100%;
        aspect-ratio: 1;
        align-self: center;
        margin:2px;
        padding:0px;
        margin: 0px;
        border-width: 2px;
        margin-right: var(--right_margin, "0px");
    }
</style>