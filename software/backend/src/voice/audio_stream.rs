use std::time::{Duration, SystemTime};

use circular_buffer::CircularBuffer;
use cpal::{StreamConfig, traits::{DeviceTrait, HostTrait, StreamTrait}};
use tokio::sync::{mpsc, watch};
use tokio_tungstenite::tungstenite::Message;

use crate::voice::{livekit_wakeword::run_wakeword_listener, whisper_streaming::run_whisper_client};

const SAMPLE_RATE:usize=16000;
const CHANNELS:u16=1;
const BITS_PER_SAMPLE:usize=16;
const CHUNK_TIME_MILLISECONDS:usize=2000;
pub const CHUNK_SIZE:usize=SAMPLE_RATE*CHUNK_TIME_MILLISECONDS/1000;
const AUDIO_STEP_SIZE:usize=CHUNK_SIZE/2;
const AUDIO_BUFFER_MULTIPLE:usize=2; //risk of stack overflow here, keep this low
const AUDIO_BUFFER_SIZE:usize=AUDIO_BUFFER_MULTIPLE*CHUNK_SIZE;
pub const CHUNK_BUFFER_MULTIPLE:usize=5;
const WEBSOCKET_MESSAGE_BUFFER_LENGTH:usize=3;

pub async fn voice_command_listener(
    wakeword_onnx_file:&str,
    url:&str
)->Result<(), Box<dyn std::error::Error + Send + Sync>>
{
    //senders and receivers
    let (wakeword_chunk_transmitter, mut wakeword_chunk_receiver) = mpsc::channel::<Box<[i16;CHUNK_SIZE]>>(CHUNK_BUFFER_MULTIPLE);
    let (last_detected_wakeword_sender, mut last_detected_wakeword_receiver) = watch::channel::<Option<SystemTime>>(None);
    let (whisper_websocket_message_transmitter, mut whisper_websocket_message_receiver) = mpsc::channel::<Message>(WEBSOCKET_MESSAGE_BUFFER_LENGTH);   

    //Audio stream initialization
    let bits_per_sample_u32:u32 = BITS_PER_SAMPLE.try_into().expect("Should convert.");
    let sample_rate_u32:u32 = SAMPLE_RATE.try_into().expect("Should convert.");
    
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

    let mut audio_buffer = CircularBuffer::<AUDIO_BUFFER_SIZE,i16>::new();
    
    let mut stream_to_whisper = false;
    let duration_to_stream_to_whisper_after_detection:Duration = Duration::from_secs(10);
    let mut last_detection:Option<SystemTime>=None;

    let data_fn = move |data: &[i16], _: &cpal::InputCallbackInfo| {
        
        //Move data to buffer
        audio_buffer.extend_from_slice(data);

        //Send data to livekit-wakeword if enough is buffered
        while audio_buffer.len()>CHUNK_SIZE
        {
            let mut chunk:Box<[i16;CHUNK_SIZE]>=Box::new([0;CHUNK_SIZE]);
            for n in 0..CHUNK_SIZE
            {
                chunk[n]=*audio_buffer.nth_front(n).expect("Should exist");
            }
            
            match wakeword_chunk_transmitter.blocking_send(chunk)
            {
                Ok(())=>{
                    //println!("Chunk sent");
                },
                Err(e)=>{
                    eprintln!("chunk send error {:?}",e);
                }
            }

            audio_buffer.truncate_front(audio_buffer.len()-AUDIO_STEP_SIZE);
        }

        //Check if wakeword detection message has changed
        match last_detected_wakeword_receiver.has_changed()
        {
            Ok(has_changed) => {
                //If it's changed, update the local copy of the value and start streaming.
                if has_changed
                {
                    last_detection=last_detected_wakeword_receiver.borrow_and_update().clone();
                    stream_to_whisper=true;

                    println!("Might want to send data in circular buffer here. Depends on how long the delay is on detection.");
                }
            },
            Err(err) => {
                eprintln!("{:?}", err);
            }
        }

        //If stream flag is true, send data to whisper
        if stream_to_whisper
        {
            let mut raw_bytes:Vec<u8>=Vec::with_capacity(data.len()*2);
            for datum in data
            {
                //Convert to float for whisper
                //This works!!
                {
                    let asfloat=if *datum < 0 {
                        *datum as f32 / 32768.0 //max for i16
                    } else {
                        *datum as f32 / 32767.0 //min for i16
                    };
                    
                    raw_bytes.extend(&asfloat.to_le_bytes());
                }
            }

            match whisper_websocket_message_transmitter.blocking_send(Message::binary(hyper::body::Bytes::from_iter(raw_bytes)))
            {
                Ok(_)=>(),
                Err(e)=>{eprintln!("{:?}",e);}
            }
            

            //Determine whether duration of whisper stream has lapsed
            match last_detection
            {
                Some(some_last_detection)=>{
                    
                    let comptime = match some_last_detection.checked_add(duration_to_stream_to_whisper_after_detection) {
                        Some(comptime)=>comptime,
                        None=>SystemTime::UNIX_EPOCH
                    };

                    if SystemTime::now()>comptime
                    {
                        stream_to_whisper=false;
                        last_detection=None;
                    }
                },
                None=>()
            }
        }
    };

    let err_fn = move |err| {
        eprintln!("An error occurred on the audio stream: {}", err);
    };

    let stream = match device.build_input_stream(
        stream_config,
        data_fn,
        err_fn,
        None, // Timeout
    ){
        Ok(stream)=>stream,
        Err(e)=>{
            eprintln!("Failed to build input stream");
            return Err(Box::new(e));
        }
    };

    match stream.play()
    {
        Ok(())=>(),
        Err(e)=>{
            eprintln!("Failed to start stream");
            return Err(Box::new(e));
        }
    }

    let wakeword_handle=run_wakeword_listener(
        wakeword_onnx_file.to_string(),
        wakeword_chunk_receiver,
        last_detected_wakeword_sender
    );
    
    let whisper_handle=run_whisper_client(
        url,
        whisper_websocket_message_receiver
    );

    tokio::join!(wakeword_handle,whisper_handle);

    Ok(())
}