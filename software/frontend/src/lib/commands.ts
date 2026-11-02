import type { Timer } from "./timer";

export type Initialization =
{
    photoprism_key:string,
    ha_key:string,
    ha_url:string
};

export type AutoTab = {
    url:string,
    priority:number,
    timeout_seconds:number
};

export type AutoTabEntry = {
    config:AutoTab,
    expiry:Date
};

export enum VoiceControlState
{
    NotEnabled,
    ListeningForWakeword,
    StreamingToWhisper
}

export type VoiceControlMode =
{
    Contains?:string
}

export type VoiceControlAction = 
{
    OpenPage?:string,
    AcknowledgeAlarms?:string
}

export type GenericVoiceControlOption =
{
    mode:VoiceControlMode|"Timer"|"Slideshow",
    action?:VoiceControlAction
}

export type VoiceControlOptions =
{
    commands:GenericVoiceControlOption[]
}

export type Command = {
    Initialization?:Initialization,
    AutoTab?:AutoTab,
    SetScreenState?:boolean,
    SetVoiceControlState?:VoiceControlState,
    AcknowledgeAlarms?:boolean,
    VoiceControlOptions?:VoiceControlOptions
    VoiceControlCurrentStream?:String,
    NewTimer?:Timer,
    ShowSlideshow?:boolean
}