use std::time::Duration;

use futures_util::{SinkExt, StreamExt, stream::SplitSink};
use serde::{Deserialize, Serialize};
use tokio::{net::TcpStream, sync::{mpsc, watch}};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async, tungstenite::Message};

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
pub struct LivekitTranscriptionWord
{
    pub word:String,
    pub start:FloatString,
    pub end:FloatString,
    pub probability:f32
}

#[derive(Serialize,Deserialize,Debug)]
pub struct LivekitTranscriptionSegment
{
    pub start:FloatString,
    pub end:FloatString,
    pub text:String,
    pub completed:bool,
    pub words:Option<Vec<LivekitTranscriptionWord>>
}

#[derive(Serialize,Deserialize,Debug)]
pub struct LivekitTranscriptionMessage
{
    pub uid:String,
    pub message:Option<String>, //will contain SERVER_READY if ready for streaming
    pub segments:Option<Vec<LivekitTranscriptionSegment>>
}

#[derive(PartialEq,Clone)]
pub enum WhisperClientControl
{
    Start,
    Stop
}

//url for testing
//let url = "ws://127.0.0.1:9090";
pub async fn run_whisper_client<T>(
    url:&str,
    mut whisper_client_control_receiver:watch::Receiver<WhisperClientControl>,
    mut websocket_message_receiver:mpsc::Receiver<Message>,
    handler_function:T
)
    where T:Fn(LivekitTranscriptionMessage)->bool
{
    loop {
        //Wait for Start command
        match whisper_client_control_receiver.wait_for(|c|{*c==WhisperClientControl::Start}).await{
            Ok(_)=>(),
            Err(e)=>{
                eprintln!("{:?}",e);
            }
        };

        //Start loop
        let handle=whisper_client_loop(
            url,
            &mut whisper_client_control_receiver,
            &mut websocket_message_receiver,
            &handler_function
        );
        tokio::join!(handle);
    }
}

async fn whisper_client_loop<T>(
    url:&str,
    whisper_client_control_receiver:&mut watch::Receiver<WhisperClientControl>,
    websocket_message_receiver:&mut mpsc::Receiver<Message>,
    handler_function:&T
)
    where T:Fn(LivekitTranscriptionMessage)->bool
{
    let (ws_stream, _response) = match connect_async(url).await
    {
        Ok((ws_stream,_response))=>{(ws_stream,_response)},
        Err(e)=>{
            eprintln!("{:?}",e);
            return;
        }
    };
    println!("WebSocket handshake has been successfully completed");

    let (mut write, mut read) = ws_stream.split();

    let livekit_config= LivekitConfig{ 
        uid: "faux_show_client".to_string(), 
        language: "en".to_string(), 
        model: "base".to_string(), 
        use_vad: true, 
        task: "transcribe".to_string(),
        //audio_format: "int16".to_string()
        audio_format: "float32".to_string(),
        word_timestamps: true,
        hotwords: "".to_string() //"supercalafragalisticexpialadocious,lymphangioleiomyomatosis".to_string()
    };
    
    println!("{:?}",livekit_config);

    //write.send(test_send.into()).await.expect("Should complete.");

    let json:String=match serde_json::to_string(&livekit_config)
    {
        Ok(json)=>json,
        Err(e)=>{
            eprintln!("{:?}",e);
            return;
        }
    };
    
    let configuration_message = Message::text(json);
    
    async fn send_message(
        write:&mut SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
        message:Message)
    {
        match write.send(message).await
        {
            Ok(())=>(),
            Err(e)=>{
                eprintln!("{:?}",e);
                return;
            }
        }
    }

    send_message(&mut write,configuration_message).await;

    //let (internal_message_transmitter, mut internal_message_receiver) = mpsc::channel::<Message>(2);   

    let websocket_message_sender = 
        async move {
            let mut continue_loop=true;
            while continue_loop {
                /*
                match websocket_message_receiver.recv().await
                {
                    Some(message) => {
                        send_message(message).await
                    },
                    None => (),
                }
                */
                
                tokio::select! {
                    Some(message)=websocket_message_receiver.recv()=>{
                        send_message(&mut write, message).await
                    },
                    result=whisper_client_control_receiver.wait_for(|c|{*c==WhisperClientControl::Stop})=>{
                        //If whisper_client_control_receiver receives a stop signal, exit the loop
                        match result
                        {
                            Ok(_) => (),
                            Err(e) => eprintln!("{:?}",e),
                        }
                        continue_loop=false;
                        match write.close().await
                        {
                            Ok(_)=>{println!("Whisper stream closed.");},
                            Err(e)=>{eprintln!("{:?}",e);}
                        };
                    }
                };
            }
        }
    ;
  
    let websocket_message_handler=
        async move {
            let mut continue_loop=true;
            while continue_loop {
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
                                                if!(handler_function(response))
                                                {
                                                    println!("Need to initiate websocket closure here!")

                                                    //Can't do it this way, doesn't stop streaming to whisper in audio_stream.rs!
                                                    //whisper_client_control_sender.send(WhisperClientControl::Stop);

                                                    //Probably need another dedicated stream
                                                }
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
                                            Some(close_frame)=>{
                                                println!("Close frame received. {:?}", close_frame);
                                            }
                                            None=>println!("Close recieved. No final frame.")
                                        };
                                        //Regardless of close frame contents, stop the loop
                                        continue_loop=false;
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

    /*
    let command_handler= async move {
        loop{
            //Wait for Stop command
            whisper_client_control_receiver.wait_for(|c|{*c==WhisperClientControl::Stop}).await;

            //And break loop
            break;
        }
    };
    */

    /*
    tokio::select!{
        _=websocket_message_sender=>{eprintln!("Whisper client message sender failed.")},
        _=websocket_message_handler=>{eprintln!("Whisper client message handler failed.")}
    }
    */

    //Graceful shutdown requires driving messages to close frames, so use a join instead of a select.
    tokio::join!{
        websocket_message_sender,
        websocket_message_handler
    };

    println!("Whisper exit.");
}