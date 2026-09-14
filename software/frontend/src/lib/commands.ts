export type TabConfig = {
    url:string,
    priority:number,
    timeout_seconds:number
};

export type AutoTabEntry = {
    config:TabConfig,
    expiry:Date
};

export enum VoiceControlState
{
    NotEnabled,
    ListeningForWakeword,
    StreamingToWhisper
}

export type Command = {
    AutoTab?:string,
    PhotoprismKey?:string,
    SetScreenState?:boolean,
    SetVoiceControlState?:VoiceControlState
}