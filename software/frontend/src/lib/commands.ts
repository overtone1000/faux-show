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

export type VoiceControlOptions =
{
    commands:{
        mode:VoiceControlMode,
        action:VoiceControlAction
    }[]
}

export type Command = {
    Initialization?:Initialization,
    AutoTab?:AutoTab,
    SetScreenState?:boolean,
    SetVoiceControlState?:VoiceControlState,
    AcknowledgeAlarms?:boolean,
    VoiceControlOptions?:VoiceControlOptions
}