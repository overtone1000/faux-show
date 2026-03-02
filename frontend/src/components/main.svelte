<script lang="ts">
    import { mdiWeatherCloudy } from '@mdi/js';
    import { mdiCctv } from '@mdi/js';
	import { mdiCog } from '@mdi/js';
    import { mdiRefresh } from '@mdi/js';
    import IconTab, { type TabProps } from './icon_tab.svelte';

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

    let tabs:TabProps[] =
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
                        url:"http://10.10.10.10:8123/dashboard-cameras/0",
                        title: "Dashboard Cameras"
                    }
                }
			},
			icon_label: "cameras",
			icon_path: mdiCctv
		},
    ];

    let refresh:TabProps = {
        action: () => {
            location.reload();
        },
        icon_label: "refresh",
        icon_path: mdiRefresh
    };
    
    tabs[1].action();

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
        height: 50px;
        display:flex;
        flex-direction: row;
    }
    .spacer
    {
        flex-shrink: true;
        width:100%;
    }
    
    /*This is for kiosks, so hide the cursor*/
    /*Doesn't work with iframe!*/
    /*
    * {
        cursor: none;
    }
    */
</style>