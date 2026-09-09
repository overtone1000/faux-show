use std::time::Duration;

use cpal::{StreamConfig, traits::{DeviceTrait, HostTrait, StreamTrait}};
use serde::{Deserialize, Serialize, de::value::Error};
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::{Bytes, Message}};
use futures_util::{SinkExt, StreamExt};


const SAMPLE_RATE:usize=16000;
const CHANNELS:u16=1;
const BITS_PER_SAMPLE:usize=16;
const WEBSOCKET_MESSAGE_BUFFER_LENGTH:usize=3;

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

#[tokio::main]
async fn main() {

    let url = "ws://127.0.0.1:9090";

    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
    println!("WebSocket handshake has been successfully completed");

    let (websocket_message_transmitter, mut websocket_message_receiver) = mpsc::channel::<Message>(WEBSOCKET_MESSAGE_BUFFER_LENGTH);
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

    //Audio stream initialization
    let bits_per_sample_u32:u32 = BITS_PER_SAMPLE.try_into().expect("Should convert.");
    let sample_rate_u32:u32 = SAMPLE_RATE.try_into().expect("Should convert.");

    //let (chunk_transmitter, mut chunk_receiver) = mpsc::channel::<Box<[i16;CHUNK_SIZE]>>(CHUNK_BUFFER_MULTIPLE);

    let host = cpal::default_host();
    let device = host.default_input_device().expect("Should exist.");

    for supported_config in device.supported_input_configs().expect("Should get configs")
    {
        if supported_config.sample_format().is_int() && 
            supported_config.sample_format().bits_per_sample()==bits_per_sample_u32 &&
            supported_config.channels() == CHANNELS &&
            supported_config.min_sample_rate() <= sample_rate_u32 &&
            supported_config.max_sample_rate() >= sample_rate_u32
        {
            println!("Desired config found.");
            println!("   {:?}",supported_config);
            break;
        }
    }

    let stream_config:StreamConfig=StreamConfig { channels: CHANNELS, sample_rate: sample_rate_u32, buffer_size: cpal::BufferSize::Default };

    //let config = device.default_input_config().expect("Should have config.");
    //println!("Default input config: {:?}", config);
    //let stream_config = config.config();
    
    let err_fn = move |err| {
        eprintln!("An error occurred on the audio stream: {}", err);
    };

    let data_fn = move |data: &[i16], _: &cpal::InputCallbackInfo| {
        let mut raw_bytes:Vec<u8>=Vec::with_capacity(data.len()*2);
        for datum in data
        {
            //raw_bytes.extend(&datum.to_le_bytes()); //Is probably little endian but not confirmed
            //raw_bytes.extend(&datum.to_be_bytes());

            
            //This works!!
            {
                let asfloat=if *datum < 0 {
                    *datum as f32 / 32768.0 //max for i16
                } else {
                    *datum as f32 / 32767.0 //min for i16
                };
                
                raw_bytes.extend(&asfloat.to_le_bytes()); //Is probably little endian but not confirmed
            }
        }

        //println!("Sending {} bytes.",raw_bytes.len());
        //println!("     {:?}",raw_bytes);
        //println!("     {:?}",data);
        websocket_message_transmitter.blocking_send(Message::binary(
                Bytes::from_iter(raw_bytes)
        )).expect("Shoud send.");
    };

    let stream = device.build_input_stream(
        stream_config,
        data_fn,
        err_fn,
        None, // Timeout
    ).expect("Failed to build input stream");

    stream.play().expect("Failed to start stream");

    let websocket_message_sender = tokio::spawn(
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
    );

    let websocket_message_handler = tokio::spawn (
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
    );

    match tokio::try_join!(websocket_message_sender,websocket_message_handler)
    {
        Ok(_) => println!("Ended gracefully."),
        Err(e) => eprintln!("Couldn't join. {:?}",e),
    }
    
    //std::thread::sleep(Duration::from_secs(10));
}