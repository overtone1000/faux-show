<script lang="ts">
	import IconButton from "./icon_button.svelte";

    export type TabProps = {
        action:()=>void,
        icon_path:string,
        icon_label:string,
        disabled?:boolean
    };

    type TabPropsCont = {props:TabProps};
    let propscont:TabPropsCont = $props();
</script>

<IconButton
    path={propscont.props.icon_path}
    label={propscont.props.icon_label}
    action={propscont.props.action}
    disabled={propscont.props.disabled}
/>

<style>
</style>