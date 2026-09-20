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

export type Command = {
    AutoTab?:AutoTab,
    PhotoprismKey?:string,
    SetScreenState?:boolean,
    SetVoiceControlState?:VoiceControlState
}