use std::{f64::consts::E, time::{Duration, SystemTime}};

use circular_buffer::CircularBuffer;
use cpal::{StreamConfig, traits::{DeviceTrait, HostTrait, StreamTrait}};
use tokio::sync::{broadcast::Receiver, mpsc::{self, UnboundedSender}, watch};
use tokio_tungstenite::tungstenite::Message;

use crate::{InitializationParameters, comm::{CommunicationHub, CommunicationSpoke, external_commands::{Command, VoiceControlState}, internal_notifications::InternalServiceNotification}, voice::{self, livekit_wakeword::run_wakeword_listener, voice_command::VoiceCommandList, whisper_streaming::{LivekitTranscriptionMessage, run_whisper_client}}};


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

pub async fn run_voice_command_listener(
    //wakeword_onnx_file:&str,
    //url:&str,
    params:&InitializationParameters,
    //external_command_sender:UnboundedSender<Command>,
    //mut internal_service_notification_receiver:Receiver<InternalServiceNotification>
    spoke:CommunicationSpoke
)->Result<(), Box<dyn std::error::Error + Send + Sync>>
{
    let mut run_voice_command:bool=true;
    let mut state_receiver=spoke.get_internal_state_receiver();
    loop {
        
        let notification_listener = async {
            //match internal_service_notification_receiver.recv().await
            match state_receiver.changed().await
            {
                Ok(()) => {
                    let state = state_receiver.borrow();
                    state.front_end_connected && !state.sleeping
                },
                Err(e) => {
                    eprintln!("{:?}",e);
                    false
                },
            }
        };

        let handle=async {
            if run_voice_command
            {
                voice_command_listener(
                    params.wakeword_onnx_file.clone(),
                    params.whisper_server_url.clone(),
                    params.config_static_directory.to_string() + "/voice_commands.json",
                    spoke.clone()
                ).await
            }
            else
            {
                std::future::pending().await
            }
        };

        /*
        match tokio::join!(handle){
            (Ok(()),)=>(),
            (Err(e),)=>{
                eprintln!("{:?}",e);
            }
        };
        */
        tokio::select! {
            run_voice_command_new_value=notification_listener=>{
                run_voice_command=run_voice_command_new_value;
            },
            result=handle=>{
                match result
                {
                    Ok(())=>{
                        println!("Audio stream returned. Restarting in one second.");
                        tokio::time::sleep(Duration::from_secs(1)).await;
                    },
                    Err(e)=>{
                        eprintln!("{:?}",e);
                        tokio::time::sleep(Duration::from_secs(10)).await;
                    }
                }
            }
        }
    }
}

pub struct WakewordWhisperState{
    pub whisper_stream_enabled:bool,
    pub last_wakeword_detection:Option<SystemTime>
}

async fn voice_command_listener(
    wakeword_onnx_file:String,
    url:String,
    config_file:String,
    spoke:CommunicationSpoke
)->Result<(), Box<dyn std::error::Error + Send + Sync>>
{
    //senders and receivers
    let (wakeword_chunk_transmitter, wakeword_chunk_receiver) = mpsc::channel::<Box<[i16;CHUNK_SIZE]>>(CHUNK_BUFFER_MULTIPLE);
    let (last_detected_wakeword_sender, last_detected_wakeword_receiver) = watch::channel::<Option<SystemTime>>(None);
    let (whisper_websocket_message_transmitter, whisper_websocket_message_receiver) = mpsc::channel::<Message>(WEBSOCKET_MESSAGE_BUFFER_LENGTH);   
    //let (whisper_websocket_control_transmitter, whisper_websocket_control_receiver) = watch::channel::<WhisperClientControl>(WhisperClientControl::Stop);   
    let (voice_control_state_sender, voice_control_state_receiver)=watch::channel::<WakewordWhisperState>(WakewordWhisperState{
        whisper_stream_enabled:false,
        last_wakeword_detection:None
    });
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

    let voice_command_list = match std::fs::read_to_string(config_file)
    {
        Ok(res)=>{
            match serde_json::from_str::<VoiceCommandList>(&res)
            {
                Ok(vcl) => vcl,
                Err(e)=>return Err(Box::new(e))
            }
        },
        Err(e)=>return Err(Box::new(e))
    };
    
    let stream_config:StreamConfig=StreamConfig { channels: CHANNELS, sample_rate: sample_rate_u32, buffer_size: cpal::BufferSize::Default };

    let mut audio_buffer = CircularBuffer::<AUDIO_BUFFER_SIZE,i16>::new();
    
    let duration_to_stream_to_whisper_after_detection:Duration = Duration::from_secs(10);
    
    //let mut stream_to_whisper = false;
    //let mut last_detection:Option<SystemTime>=None;
    
    let (start_whisper, stop_whisper)={

            
        fn set_state(state:WakewordWhisperState, 
            voice_control_state_sender:&watch::Sender<WakewordWhisperState>
        )
        {
            match voice_control_state_sender.send(state){
                Ok(())=>(),
                Err(e)=>eprintln!("{:?}",e)
            };
        }

        fn start_streaming(
            voice_control_state_sender:&watch::Sender<WakewordWhisperState>,
            last_detected_wakeword_receiver:&mut watch::Receiver<Option<SystemTime>>
        )
        {
            set_state(
                WakewordWhisperState{
                    whisper_stream_enabled:true,
                    last_wakeword_detection:last_detected_wakeword_receiver.borrow_and_update().clone()
                },
                voice_control_state_sender
            );
        }

        fn stop_streaming(
            voice_control_state_sender:&watch::Sender<WakewordWhisperState>
        )
        {
            set_state(
                WakewordWhisperState{
                    whisper_stream_enabled:false,
                        last_wakeword_detection:None
                },
                voice_control_state_sender
            );
        }

        let spoke_clone_1 = spoke.clone();
        let spoke_clone_2 = spoke.clone();
        let voice_control_state_sender_clone_1 = voice_control_state_sender.clone();
        let voice_control_state_sender_clone_2 = voice_control_state_sender.clone();
        //let state_clone = state.clone();
        let mut last_detected_wakeword_receiver_clone = last_detected_wakeword_receiver.clone();

        let start_whisper=move||{
            start_streaming(&voice_control_state_sender_clone_1, &mut last_detected_wakeword_receiver_clone);
            spoke_clone_1.clone().send_external_command(Command::SetVoiceControlState(VoiceControlState::StreamingToWhisper));
            //println!("Might want to send data in circular buffer here. Depends on how long the delay is on detection.");
            //Probably not
        };

        let stop_whisper=move || {
            println!("Stopping stream to whisper.");
            stop_streaming(&voice_control_state_sender_clone_2);
            spoke_clone_2.send_external_command(Command::SetVoiceControlState(VoiceControlState::ListeningForWakeword));
        };
        (start_whisper, stop_whisper)
    };
   
    let data_fn = {
        let mut start_whisper_clone=start_whisper.clone();        
        let stop_whisper_clone = stop_whisper.clone();     
        let mut voice_control_state_receiver_clone = voice_control_state_receiver.clone();
        let mut last_detected_wakeword_receiver_clone = last_detected_wakeword_receiver.clone();

        move |data: &[i16], _: &cpal::InputCallbackInfo| {

            /*
            let set_whisper_control_mode=|mode:WhisperClientControl|
            {
                match whisper_websocket_control_transmitter.send(mode.clone())
                {
                    Ok(())=>(),
                    Err(e)=>{
                        eprintln!("{:?}",e);
                    }
                };
            };
            */

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
            match last_detected_wakeword_receiver_clone.has_changed()
            {
                Ok(has_changed) => {
                    last_detected_wakeword_receiver_clone.mark_unchanged();
                    //If it's changed, update the local copy of the value and start streaming.
                    if has_changed
                    {
                        println!("Wakeword detection changed. Starting whisper.");
                        /*
                        last_detection=last_detected_wakeword_receiver.borrow_and_update().clone();
                        set_whisper_control_mode(WhisperClientControl::Start);
                        stream_to_whisper=true;
                        spoke_clone.send_external_command(Command::SetVoiceControlState(VoiceControlState::StreamingToWhisper));
                        println!("Might want to send data in circular buffer here. Depends on how long the delay is on detection.");
                        */
                        //start_streaming(&voice_control_state_sender_clone, &mut last_detected_wakeword_receiver_clone);
                        start_whisper_clone();
                    }
                },
                Err(err) => {
                    eprintln!("{:?}", err);
                }
            }

            let state=voice_control_state_receiver_clone.borrow_and_update();
            if state.whisper_stream_enabled
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
                match state.last_wakeword_detection
                {
                    Some(some_last_detection)=>{
                        
                        let comptime = match some_last_detection.checked_add(duration_to_stream_to_whisper_after_detection) {
                            Some(comptime)=>comptime,
                            None=>SystemTime::UNIX_EPOCH
                        };

                        if SystemTime::now()>comptime
                        {
                            /*
                            println!("Stopping stream to whisper.");
                            set_whisper_control_mode(WhisperClientControl::Stop);
                            stream_to_whisper=false;
                            spoke_clone.send_external_command(Command::SetVoiceControlState(VoiceControlState::ListeningForWakeword));
                            last_detection=None;
                            */
                            stop_whisper_clone();
                        }
                    },
                    None=>()
                }
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
        Ok(())=>{},
        Err(e)=>{
            eprintln!("Failed to start stream");
            return Err(Box::new(e));
        }
    }

    let handler_function = {   
        let stop_whisper_clone = stop_whisper.clone();   
        let spoke_clone=spoke.clone();  
            move|message:LivekitTranscriptionMessage|{
            match message.message
            {
                Some(message)=>{
                    println!("Got message: {}",message);
                    match message.as_str()
                    {
                        "SERVER_READY"=>{
                            println!("Whisper is ready.");
                        }
                        _=>()
                    };
                },
                None=>()
            };

            match message.segments
            {
                Some(segments)=>{
                    println!("There are {} segments.",&segments.len());
                    for segment in &segments
                    {
                        let c=match segment.completed
                        {
                            true=>"Completed",
                            false=>"Incomplete"
                        };
                        println!("   {}:{}",c,segment.text);
                    }

                    let finished=voice_command_list.check_for_match_and_run_best_match(&segments, &spoke_clone);
                    if finished
                    {
                        //If there is a match (check returns false), stop the whisper stream
                        stop_whisper_clone();
                    }
                },
                None=>()
            }
        }
    };

    let wakeword_handle=run_wakeword_listener(
        wakeword_onnx_file.to_string(),
        wakeword_chunk_receiver,
        last_detected_wakeword_sender
    );
    
    let whisper_handle=run_whisper_client(
        &url,
        voice_control_state_receiver,
        whisper_websocket_message_receiver,
        handler_function
    );

    spoke.send_external_command(Command::SetVoiceControlState(VoiceControlState::ListeningForWakeword));
    tokio::join!(wakeword_handle,whisper_handle);

    println!("Audio stream joined.");

    spoke.send_external_command(Command::SetVoiceControlState(VoiceControlState::NotEnabled));
    Ok(())
}