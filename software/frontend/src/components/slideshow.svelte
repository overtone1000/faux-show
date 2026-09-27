<script lang="ts">
	import { get_album_by_uid, get_all_albums, type Album } from "$lib/photoprism/albums";
	import { get_download_token } from "$lib/photoprism/commons";
    //import { DEVKEY } from "$lib/photoprism/devsecrets";
	import { download_photo, get_random_photo_uid_from_album, type Photo } from "$lib/photoprism/photos";
	import { onDestroy, onMount } from "svelte";
    import { fly } from 'svelte/transition';

    console.debug("Initializing slideshow.");

    export type SlideshowProps =
    {
        photoprism_key:string|undefined
    };

    let props:SlideshowProps = $props();

    console.warn("Need to make these configurable via environment variables.");
    const BASE="https://photos.overdesigned.org/api/v1";
    const ALBUM_UID="atbl6hj66z1i4hxf";
   

    let images:{blob:Blob,url:string}[] = [];
    let image_pointer:number|undefined=$state(undefined);
    let last_image_pointer:number|undefined=$derived.by(
        ()=>{
            if(image_pointer===0)
            {
                return 1;
            }
            else
            {
                return 0;
            }
        }
    );

    const update_interval=30000;
    const min_update_interval=1000;
    //const update_interval=5000;
    let last_photo_offset:number=-1;
    let last_successful_update:number=0;
    
    async function update_steps(key:string)
    {
        console.debug("Beginning update_steps");

        const album:(Album|null)=await get_album_by_uid(ALBUM_UID,BASE,key);
        if(album===null){console.debug("album is null");return;}
        const photo_result=await get_random_photo_uid_from_album(album,BASE,key,last_photo_offset);
        if(photo_result===null){console.debug("photo_result is null");return;}
        const download_token:(string|null)=await get_download_token(BASE,key);
        if(download_token===null){console.debug("download_token is null");return;}
        const downloaded_photo:(Blob|null)=await download_photo(photo_result.photo,BASE,key,download_token,update_interval);
        if(downloaded_photo===null){console.debug("downloaded_photo is null");return;}

        //This won't work because authentication header is necessary for API.
        //console.debug("Maybe try just setting src URL directly to download URL?")

        console.debug("Got photo. Size: " + downloaded_photo.size + ", type: " + downloaded_photo.type);

        last_photo_offset=photo_result.last_offset;
        const new_image = {blob:downloaded_photo,url:URL.createObjectURL(downloaded_photo)};

        console.debug("Photo URL: " + new_image.url);

        if(images.length<1)
        {
            images.push(new_image);
            image_pointer=images.length-1;
        }
        else
        {
            let target:number;
            if(image_pointer===0)
            {
                target=1;
            }
            else
            {
                target=0;
            }
            images[target]=new_image;
            image_pointer=target;
        }

        last_successful_update=Date.now();
        console.debug("Finished update_steps");
    }

    //let last_update:string=$state("");
    let current_timeout:number|undefined=undefined;
    async function update_image()
    {
        console.debug("Beginning update_image");
        //last_update=Date.now().toString() + " " + props.photoprism_key;
        try
        {
            const KEY=props.photoprism_key;
            //const KEY=DEVKEY; //Enable for rapid development
            if(KEY!==undefined)
            {
                await update_steps(KEY);
            }
            else
            {
                console.debug("Key undefined.");
            }
        }
        catch(error)
        {
            console.error(error);
        }
        finally
        {
            console.debug("update_image finally");
            const time_since_last_success=Date.now()-last_successful_update;
            let new_timeout=update_interval-time_since_last_success;

            if(new_timeout>update_interval){new_timeout=update_interval;}
            else if(new_timeout<min_update_interval){new_timeout=min_update_interval;}
            console.debug("Setting new timeout in "+ new_timeout + "ms");
            current_timeout=setTimeout(update_image,new_timeout);
        }
        console.debug("Exiting update_image");
    }

    //let interval_id:number;
    onMount(()=>{
        console.debug("Mounting.");
        update_image();
        //interval_id=setInterval(update_image,millis_until_next_image);
        console.debug("Mounted.");
    });

    onDestroy(()=>{
        console.debug("Destroying.");
        //clearInterval(interval_id);
        clearTimeout(current_timeout);
        console.debug("Destroyed.");
    });

    //const FADE={delay:0,duration:1500};
    const FLY_IN={x:200,duration:3000}
    const FLY_OUT={x:-200,duration:3000}

    console.debug("Finished slideshow initialization.");
</script>

<div class="outer">
    {#if image_pointer!==undefined && images.length>=1 && image_pointer===0}
        <img in:fly={FLY_IN} out:fly={FLY_OUT} class="image" alt="Slideshow" src={images[0].url} />
    {:else if image_pointer!==undefined && images.length>=2 && image_pointer===1}
        <img in:fly={FLY_IN} out:fly={FLY_OUT} class="image" alt="Slideshow" src={images[1].url} />
    {/if}
</div>

<style>
    .outer
    {
        width:100%;
        height:100%;
        min-width:0%;
        min-height:0%;
        flex-grow: 1;
        flex-shrink: 1;
        overflow-y: hidden;
        overflow-x: hidden;
        display: grid;
        grid-template-columns: 100%;
        grid-template-rows: 100%;
        place-items: center;
    }
    .image{
        grid-area: 1 / 1;
        max-width:100%;
        max-height:100%;
        height:auto;
        width:auto;
    }
</style>