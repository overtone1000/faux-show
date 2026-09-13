use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};

#[derive(Serialize,Debug)]
struct LivekitConfig
{
    uid:String, //arbitrary
    language:String, //"en"
    model:String,
    use_vad:bool, //discard audio that doesn't seem to contain voice info
    task:String, //transcribe or translate
    audio_format:String, //float32, int16, or uint8
    word_timestamps:bool, //shows individual word probabilities while a segment is incomplete
    hotwords:String //comma separated list of words that the model should expect
}

#[derive(Debug)]
struct FloatString
{
    value:f32
}
impl <'de> Deserialize<'de> for FloatString
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de> {
            match String::deserialize(deserializer)
            {
                Ok(str) => {
                    match str.parse::<f32>()
                    {
                        Ok(f32)=>Ok(FloatString{value:f32}),
                        Err(e)=>{
                            eprintln!("{:?}",e);
                            Ok(FloatString{value:0.0})
                        }
                    }
                },
                Err(e) => Err(e),
            }
    }
}
impl Serialize for FloatString
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer {
        self.value.to_string().serialize(serializer)
    }
}

#[derive(Serialize,Deserialize,Debug)]
struct LivekitTranscriptionWord
{
    word:String,
    start:FloatString,
    end:FloatString,
    probability:f32
}

#[derive(Serialize,Deserialize,Debug)]
struct LivekitTranscriptionSegment
{
    start:FloatString,
    end:FloatString,
    text:String,
    completed:bool,
    words:Option<Vec<LivekitTranscriptionWord>>
}

#[derive(Serialize,Deserialize,Debug)]
struct LivekitTranscriptionMessage
{
    uid:String,
    message:Option<String>, //will contain SERVER_READY if ready for streaming
    segments:Option<Vec<LivekitTranscriptionSegment>>
}

//url for testing
//let url = "ws://127.0.0.1:9090";
pub async fn run_whisper_client(
    url:&str,
    mut websocket_message_receiver:mpsc::Receiver<Message>
) {
    loop {
        let handle=whisper_client_loop(url, &mut websocket_message_receiver);
        tokio::join!(handle);
    }
}

async fn whisper_client_loop(
    url:&str,
    websocket_message_receiver:&mut mpsc::Receiver<Message>
) {
    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
    println!("WebSocket handshake has been successfully completed");

    let (mut write, mut read) = ws_stream.split();

    let test_send= LivekitConfig{ 
        uid: "test_client".to_string(), 
        language: "en".to_string(), 
        model: "base".to_string(), 
        use_vad: true, 
        task: "transcribe".to_string(),
        //audio_format: "int16".to_string()
        audio_format: "float32".to_string(),
        word_timestamps: true,
        hotwords: "".to_string() //"supercalafragalisticexpialadocious,lymphangioleiomyomatosis".to_string()
    };
    
    println!("{:?}",test_send);

    //write.send(test_send.into()).await.expect("Should complete.");

    let json:String=serde_json::to_string(&test_send).expect("Should deserialize.");
    let test_message = Message::text(json);
    
    write.send(test_message).await.expect("Should complete.");

    let websocket_message_sender = 
        async move {
            loop {
                match websocket_message_receiver.recv().await
                {
                    Some(message) => {
                        //println!("Sending message. {:?}",message);
                        write.send(message).await.expect("Should send.")
                    },
                    None => (),
                }
            }
        }
    ;

    let websocket_message_handler=
        async move {
            loop {
                match read.next().await
                {
                    Some(next) => {
                        match next
                        {
                            Ok(next)=>{ 
                                match next
                                {
                                    Message::Text(utf8_bytes) => {
                                        match serde_json::from_slice::<LivekitTranscriptionMessage>(utf8_bytes.as_bytes())
                                        {
                                            Ok(response)=>{
                                                println!("{:?}",response);
                                            },
                                            Err(e)=>{
                                                eprintln!("{:?}",e);
                                            }
                                        }
                                    },
                                    Message::Binary(_bytes) => (),
                                    Message::Ping(_bytes) => (),
                                    Message::Pong(_bytes) => (),
                                    Message::Close(close_frame) => {
                                        match close_frame
                                        {
                                            Some(_close_frame)=>(),
                                            None=>()
                                        }
                                    },
                                    Message::Frame(_frame) => (),
                                }
                            },
                            Err(e)=>{
                                eprintln!("{:?}",e);
                            }
                        }
                    },
                    None => (),
                }
            }
        }
    ;

    tokio::select!{
        _=websocket_message_sender=>{eprintln!("Whisper client message sender failed.")},
        _=websocket_message_handler=>{eprintln!("Whisper client message handler failed.")}
    }
}