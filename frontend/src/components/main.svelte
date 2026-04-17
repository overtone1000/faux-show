<script lang="ts">
    import { mdiClock } from '@mdi/js';
    import { mdiRefresh } from '@mdi/js';
    import IconTab, { type TabProps } from './icon_tab.svelte';
	import { onMount } from 'svelte';
	import Time from './time.svelte';
	import TimerPage, { type Timer, type TimerState as TimerState } from './timer_page.svelte';
	import type { Command } from '$lib/commands';

    console.debug("Start init.");

    enum MainField {
        iframe,
        component
    };

    type IFrameMeta = {
        url:string,
        title:string
    };

    enum ComponentType {
        clock
    };

    type Main = {
        field:MainField,
        iframe_meta?:IFrameMeta
        component_meta?:ComponentType
    };

    let main:Main|undefined = $state(undefined);

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
        icon_path: mdiClock
    };

    function handle_server_command(command:Command)
    {
        console.debug("Handling command.");
        if(command.ChangeDash)
        {
            console.debug("Changing dash to " + command.ChangeDash.index);
            if(tabs !== undefined && tabs[command.ChangeDash.index] !== undefined)
            {
                tabs[command.ChangeDash.index].action();
            }
        }
    }

    function open_socket(){
        const socket_url = "ws:/"+location.host;
        console.debug("Opening websocket on");
        const socket = new WebSocket(socket_url);

        //const test = () => {
        //    console.debug("Test...");
        //    socket.send(new Date().toString());
        //    setTimeout(test,1000);
        //}

        // Connection opened
        socket.addEventListener("open", (event) => {
            console.debug("Connection opened.");
            //test();
        });

        // Listen for messages
        socket.addEventListener("message", (event) => {
            console.log("Message from server ", event.data);
            handle_server_command(JSON.parse(event.data));
        });
    };

    type TabsConfig = {
        label:string,
        title:string,
        icon_path:string,
        url:string
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
            tabs[0].action();
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

    console.debug("End init.");

</script>

<div class="main">
    <div class="tab-row">
        {#each tabs as tab}
            <IconTab --right_margin="4px" props={tab}/>
        {/each}
        <IconTab props={clock}/>
        <div class="spacer"></div>
        <Time/>
        <div class="spacer"></div>
        <IconTab props={refresh}/>
    </div>
    {#if main !== undefined}
        {#if main.field === MainField.iframe && main.iframe_meta !== undefined}
            <iframe class="full-width" src={main.iframe_meta.url} title={main.iframe_meta.title}>
                <p>iframe unsupported</p>
            </iframe>
            <div class="hide-cursor"></div>
        {:else if main.field === MainField.component && main.component_meta !== undefined}
            {#if main.component_meta === ComponentType.clock}
                <TimerPage timers={timers}/>
            {/if}
        {/if}
    {/if}
</div>

<style>
    .full-width
    {
        width:100%;
        flex-grow: 1;
    }
    .main
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
    * {
        color-scheme: dark;
    }
    /*This is for kiosks, so hide the cursor*/
    /*Doesn't work with iframe!*/
    /*
    * {
        cursor: none;
    }
    */
</style>