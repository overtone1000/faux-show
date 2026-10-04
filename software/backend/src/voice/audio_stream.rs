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

fn get_state(voice_control_state_receiver:&mut watch::Receiver<WakewordWhisperState>)->WakewordWhisperState
{
    //When getting state, it's best to clone the value to release the borrow, especially if subsequent async tasks are performed.
    (*(voice_control_state_receiver.borrow_and_update())).clone()
}

fn set_state(state:WakewordWhisperState, 
    voice_control_state_sender:&watch::Sender<WakewordWhisperState>
)
{
    println!("audio_stream.rs: Setting state {:?}",state);
    match voice_control_state_sender.send(state){
        Ok(())=>println!("audio_stream.rs: State set."),
        Err(e)=>eprintln!("audio_stream.rs: {:?}",e)
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

pub fn get_voice_command_json_file_path(params:&InitializationParameters)->String
{
    params.config_static_directory.to_string() + "/voice_commands.json"
}

pub async fn run_voice_command_listener(
    //wakeword_onnx_file:&str,
    //url:&str,
    params:&InitializationParameters,
    //external_command_sender:UnboundedSender<Command>,
    //mut internal_service_notification_receiver:Receiver<InternalServiceNotification>
    spoke:CommunicationSpoke
)->Result<(), Box<dyn std::error::Error + Send + Sync>>
{
    let mut voice_command_enabled:bool=true;
    let mut state_receiver=spoke.get_internal_state_receiver();
    loop {
        
        let internal_state_voice_command_enabled = async {
            //match internal_service_notification_receiver.recv().await
            match state_receiver.changed().await
            {
                Ok(()) => {
                    let state = state_receiver.borrow();
                    state.front_end_connected && !state.sleeping
                },
                Err(e) => {
                    eprintln!("audio_stream.rs: {:?}",e);
                    false
                },
            }
        };

        let handle=async {
            if voice_command_enabled
            {
                voice_command_listener(
                    params.wakeword_onnx_file.clone(),
                    params.whisper_server_url.clone(),
                    get_voice_command_json_file_path(params),
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
                eprintln!("audio_stream.rs: {:?}",e);
            }
        };
        */
        tokio::select! {
            voice_command_enabled_new_value=internal_state_voice_command_enabled=>{
                voice_command_enabled=voice_command_enabled_new_value;
            },
            result=handle=>{
                match result
                {
                    Ok(())=>{
                        println!("audio_stream.rs: Audio stream returned. Restarting in one second.");
                        tokio::time::sleep(Duration::from_secs(1)).await;
                    },
                    Err(e)=>{
                        eprintln!("audio_stream.rs: {:?}",e);
                        tokio::time::sleep(Duration::from_secs(10)).await;
                    }
                }
            }
        }
    }
}

#[derive(Debug,Clone)]
pub struct WakewordWhisperState{
    pub whisper_stream_enabled:bool,
    pub last_wakeword_detection:Option<SystemTime>
}

async fn voice_command_listener(
    wakeword_onnx_file:String,
    url:String,
    voice_config_file:String,
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
            println!("audio_stream.rs: Desired config found.");
            println!("audio_stream.rs:    {:?}",supported_config);
            break;
        }
    }

    let voice_command_list = match std::fs::read_to_string(voice_config_file)
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

         

        let spoke_clone_1 = spoke.clone();
        let spoke_clone_2 = spoke.clone();
        let voice_control_state_sender_clone_1 = voice_control_state_sender.clone();
        let voice_control_state_sender_clone_2 = voice_control_state_sender.clone();
        //let state_clone = state.clone();
        let mut last_detected_wakeword_receiver_clone = last_detected_wakeword_receiver.clone();
        //let mut last_detected_wakeword_receiver_clone_testing = last_detected_wakeword_receiver.clone();

        let start_whisper=move||{
            println!("audio_stream.rs: Sending command to start whisper.");
            start_streaming(&voice_control_state_sender_clone_1, &mut last_detected_wakeword_receiver_clone);
            spoke_clone_1.clone().send_external_command(Command::SetVoiceControlState(VoiceControlState::StreamingToWhisper));
            //println!("audio_stream.rs: Might want to send data in circular buffer here. Depends on how long the delay is on detection.");
            //Probably not
        };

        let stop_whisper=move || {
            eprintln!("audio_stream.rs: This isn't closing whisper correctly. Perhaps async is stuck somewhere or state isn't being set correctly.");
            println!("audio_stream.rs: Sending stop signal.");
            stop_streaming(&voice_control_state_sender_clone_2); //This is definite the problem here!
            
            //Even this fails! This must be an odd async bug.
            //start_streaming(&voice_control_state_sender_clone_2, &mut last_detected_wakeword_receiver_clone_testing); //For testing only

            //This is working fine
            println!("audio_stream.rs: Setting state.");
            spoke_clone_2.send_external_command(Command::SetVoiceControlState(VoiceControlState::ListeningForWakeword));
            println!("audio_stream.rs: Finished stopping whisper");
        };
        (start_whisper, stop_whisper)
    };
   
    let data_fn = {
        let mut start_whisper_clone=start_whisper.clone();        
        let stop_whisper_clone = stop_whisper.clone();     
        let mut voice_control_state_receiver_clone = voice_control_state_receiver.clone();
        let mut last_detected_wakeword_receiver_clone = last_detected_wakeword_receiver.clone();
        let whisper_websocket_message_transmitter_clone = whisper_websocket_message_transmitter.clone();

        move |data: &[i16], _: &cpal::InputCallbackInfo| {
            //Move data to buffer
            audio_buffer.extend_from_slice(data);

            //Send data to livekit-wakeword if enough is buffered
            while audio_buffer.len()>CHUNK_SIZE
            {
                //println!("audio_stream.rs: data_function: Audio buffer still running.");
                let mut chunk:Box<[i16;CHUNK_SIZE]>=Box::new([0;CHUNK_SIZE]);
                for n in 0..CHUNK_SIZE
                {
                    chunk[n]=*audio_buffer.nth_front(n).expect("Should exist");
                }
                
                //println!("audio_stream.rs: data_function: Sending chunk");
                match wakeword_chunk_transmitter.blocking_send(chunk)
                {
                    Ok(())=>{
                        //println!("audio_stream.rs: data_function: Chunk sent");
                    },
                    Err(e)=>{
                        eprintln!("audio_stream.rs: data_function: chunk send error {:?}",e);
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
                        println!("audio_stream.rs: data_function: Wakeword detection changed. Starting whisper.");
                        /*
                        last_detection=last_detected_wakeword_receiver.borrow_and_update().clone();
                        set_whisper_control_mode(WhisperClientControl::Start);
                        stream_to_whisper=true;
                        spoke_clone.send_external_command(Command::SetVoiceControlState(VoiceControlState::StreamingToWhisper));
                        println!("audio_stream.rs: Might want to send data in circular buffer here. Depends on how long the delay is on detection.");
                        */
                        //start_streaming(&voice_control_state_sender_clone, &mut last_detected_wakeword_receiver_clone);
                        start_whisper_clone();
                    }
                },
                Err(err) => {
                    eprintln!("audio_stream.rs: data_function: {:?}", err);
                }
            }

            //println!("Getting state.");
            let state=get_state(&mut voice_control_state_receiver_clone);
            //println!("Got state.");
            //Isn't failing here.
            if state.whisper_stream_enabled
            {
                //println!("audio_stream.rs: data_function: Whisper stream still running.");
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

                //println!("audio_stream.rs: data_function: Sending to whisper websocket.");
                match whisper_websocket_message_transmitter_clone.blocking_send(Message::binary(hyper::body::Bytes::from_iter(raw_bytes)))
                {
                    Ok(_)=>(),
                    Err(e)=>eprintln!("audio_stream.rs: data_function: {:?}",e)
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
                            stop_whisper_clone();
                        }
                    },
                    None=>()
                }

                //println!("audio_stream.rs: data_function: Whisper segment completed.");
            }
        }
    };

    let err_fn = move |err| {
        eprintln!("audio_stream.rs: An error occurred on the audio stream: {}", err);
    };

    let stream = match device.build_input_stream(
        stream_config,
        data_fn,
        err_fn,
        None, // Timeout
    ){
        Ok(stream)=>stream,
        Err(e)=>{
            eprintln!("audio_stream.rs: Failed to build input stream");
            return Err(Box::new(e));
        }
    };

    match stream.play()
    {
        Ok(())=>{},
        Err(e)=>{
            eprintln!("audio_stream.rs: Failed to start stream");
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
                    println!("audio_stream.rs: Got message: {}",message);
                    match message.as_str()
                    {
                        "SERVER_READY"=>{
                            println!("audio_stream.rs: Whisper is ready.");
                        }
                        _=>()
                    };
                },
                None=>()
            };

            match message.segments
            {
                Some(segments)=>{
                    println!("audio_stream.rs: There are {} segments.",&segments.len());
                    for segment in &segments
                    {
                        let c=match segment.completed
                        {
                            true=>"Completed",
                            false=>"Incomplete"
                        };
                        println!("audio_stream.rs:    {}:{}",c,segment.text);
                    }

                    let finished=voice_command_list.check_for_match_and_run_best_match(&segments, &spoke_clone);
                    if finished
                    {
                        //If there is a match (check returns false), stop the whisper stream
                        stop_whisper_clone();
                    }
                },
                None=>()
            };
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

    tokio::select!{
        _=wakeword_handle=>{eprintln!("audio_stream.rs: Wakeword loop exit")},
        _=whisper_handle=>{eprintln!("audio_stream.rs: Whisper loop exit")}
    }
    //tokio::join!(wakeword_handle,whisper_handle);

    println!("audio_stream.rs: Audio stream joined.");
    spoke.send_external_command(Command::SetVoiceControlState(VoiceControlState::NotEnabled));
    Ok(())
}