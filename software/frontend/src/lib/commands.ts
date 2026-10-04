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

export type VoiceControlModeContains =
{
    Contains:string
}

export type VoiceControlActionOpenPage = 
{
    OpenPage:string
}

export type VoiceControlActionAcknowledgeAlarms = 
{
    AcknowledgeAlarms:string
}

export type VoiceControlOptions =
{
    mode:VoiceControlModeContains,
    action:VoiceControlActionOpenPage|VoiceControlActionAcknowledgeAlarms
}[]

export type Command = {
    AutoTab?:AutoTab,
    PhotoprismKey?:string,
    HASKey?:string,
    SetScreenState?:boolean,
    SetVoiceControlState?:VoiceControlState,
    AcknowledgeAlarms?:boolean,
    VoiceControlOptions?:VoiceControlOptions
}