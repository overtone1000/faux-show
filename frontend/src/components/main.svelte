<script lang="ts">
    import { mdiWeatherCloudy } from '@mdi/js';
    import { mdiCctv } from '@mdi/js';
	import { mdiCog } from '@mdi/js';
    import { mdiRefresh } from '@mdi/js';
    import IconTab, { type TabProps } from './icon_tab.svelte';
	import { onMount } from 'svelte';

    console.debug("Start init.");

    enum MainField {
        iframe
    };

    type IFrameMeta = {
        url:string,
        title:string
    };

    type Main = {
        field:MainField,
        meta:IFrameMeta|undefined
    };

    let main:Main|undefined = $state(undefined);

    let tabs:TabProps[]|undefined = $state(undefined);
    
    const refresh:TabProps = {
        action: () => {
            location.reload();
        },
        icon_label: "refresh",
        icon_path: mdiRefresh
    };

    function open_socket(){
        const socket_url = "ws:/"+location.host;
        console.debug("Opening websocket on");
        const socket = new WebSocket(socket_url);

        // Connection opened
        socket.addEventListener("open", (event) => {
            console.debug("Connection opened.");
            socket.send("Hello Server!");
        });

        // Listen for messages
        socket.addEventListener("message", (event) => {
            console.log("Message from server ", event.data);
        });
    };

    type TabsConfig = {
        label:string,
        title:string,
        icon_path:string,
        url:string
    };

    function build_tabs(tabs_config:TabsConfig[]) {
        /*
            [
                {
                    action: function (): void {
                        throw new Error('Function not implemented.');
                    },
                    icon_label: "weather",
                    icon_path: mdiWeatherCloudy
                },
                {
                    action:  () => {
                        console.debug("cameras");
                        main={
                            field: MainField.iframe,
                            meta:{
                                url:"http://10.10.10.10:8123/dashboard-kiosk/1",
                                title: "Dashboard Cameras"
                            }
                        }
                    },
                    icon_label: "cameras",
                    icon_path: mdiCctv
                },
            ];
        */

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
                                meta:{
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
            <IconTab props={tab}/>
        {/each}
        <div class="spacer"></div>
        <IconTab props={refresh}/>
    </div>
    {#if main !== undefined}
        {#if main.field === MainField.iframe && main.meta !== undefined}
            <iframe class="full-width" src={main.meta.url} title={main.meta.title}>
                <p>iframe unsupported</p>
            </iframe>
            <div class="hide-cursor"></div>
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
        height: 5%;
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