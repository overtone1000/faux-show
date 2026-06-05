<script lang="ts">
    import { mdiCheck, mdiCircleOffOutline, mdiCircleOutline, mdiClock, mdiCross, mdiDebugStepInto, mdiImageMultiple } from '@mdi/js';
    import { mdiRefresh } from '@mdi/js';
    import { mdiRobot } from '@mdi/js';
    import IconTab, { type TabProps } from './icon_tab.svelte';
	import { onMount } from 'svelte';
	import Time from './time.svelte';
	import TimerPage, { type Timer, type TimerState as TimerState } from './timer_page.svelte';
	import type { AutoTabEntry, Command, TabConfig } from '$lib/commands';
	import Slideshow from './slideshow.svelte';
	import type { LegacyComponentType } from 'svelte/legacy';
	import IconSvg from './icon_svg.svelte';

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

    const timers:Timer[] = $state([]);

    let tabs:TabProps[]|undefined = $state(undefined);
    
    const refresh:TabProps = {
        action: () => {
            location.reload();
        },
        icon_label: "refresh",
        icon_path: mdiRefresh
    };

    const clock:TabProps = {
        action: () => {
            main={
                field: MainField.component,
                component_meta:ComponentType.clock
            }
        },
        icon_label: "clock",
        icon_path: mdiClock,
        disabled: true //Not ready yet
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
    const debug:TabProps = {
        action: () => {
            show_debug=!show_debug;
        },
        icon_label: "debug",
        icon_path: mdiDebugStepInto,
        disabled: false
    }

    function set_manual_tab(tab_props:TabProps)
    {
        manual_tab_props=tab_props;
        active_tab_props=tab_props;
        //tab_props.action(); //Don't need to do this, it will run in the effect below.
    }

    let manual_tab_props=$state<TabProps|null>(null);
    let auto_tab_config=$state<TabConfig|null>(null);

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
                if(selected_config===undefined || 
                    auto_tab_entry.config.priority>selected_config.config.priority ||
                    (
                        auto_tab_entry.config.priority==selected_config.config.priority &&
                        auto_tab_entry.expiry<selected_config.expiry
                    )
                )
                {
                    selected_config=auto_tab_entry;
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
    function handle_server_command(command:Command)
    {
        console.debug("Handling command.");
        if(command.AutoTab)
        {
            let auto_tab:TabConfig=JSON.parse(command.AutoTab);
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
        
        if(command.SetScreenState!==undefined)
        {
            display_on=command.SetScreenState
        }
    }

    let socket:WebSocket|undefined=undefined;
    let socket_state:boolean=$state(false);
    function open_socket(){
        const socket_url = "ws:/"+location.host;
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
            console.error(error.message);
        }
    }

    onMount(()=>{
        open_socket();
        get_tabs();
    });

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
            <IconSvg path={mdiCircleOutline} stroke="green"/>
            </div>
        {:else}
            <div class="infotab">
            <IconSvg path={mdiCircleOffOutline} stroke="red"/>
            </div>
        {/if}
        <IconTab props={debug}/>
        <IconTab props={refresh}/>
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
                        <TimerPage timers={timers}/>
                    {:else if main.component_meta === ComponentType.slideshow}
                        <Slideshow photoprism_key={photoprism_key}/>
                    {/if}
                {/if}
            </div>
        {/if}
        {#if show_debug}
            <div class="console">
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
    .console
    {
        grid-area: 1 / 1;
        max-width:100%;
        max-height:100%;
        height:100%;
        width:100%;
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