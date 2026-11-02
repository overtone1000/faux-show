use std::time::{Duration, SystemTime};

use livekit_wakeword::{SAMPLE_RATE, wakeword::WakeWordModel};

use tokio::sync::{mpsc, watch};

use crate::voice::{audio_stream::CHUNK_SIZE, voice_command::VoiceCommandList};

//const THRESHOLD:f32=0.89;
//Analysis says optimal threshold is 0.7 fore faux_show_v2 and 0.58 for hey_faux_show_v1
//const THRESHOLD:f32=0.7;

//Livekit wakeword *MUST* be run in release mode or it is very slow. It also uses quite a bit of CPU.

//Make chunk sender and chunk receiver pair with
//let (chunk_transmitter, mut chunk_receiver) = mpsc::channel::<Box<[i16;CHUNK_SIZE]>>(CHUNK_BUFFER_MULTIPLE);

pub async fn run_wakeword_listener(
    wakeword_onnx_file:String,
    mut chunk_receiver:mpsc::Receiver<Box<[i16;CHUNK_SIZE]>>,
    last_detected_wakeword_sender:watch::Sender<Option<SystemTime>>,
    wakeword_threshold:f32
)
{
    loop {
        let handle=wakeword_listener_loop(
            &wakeword_onnx_file, 
            &mut chunk_receiver,
            &last_detected_wakeword_sender,
            wakeword_threshold
        );
        tokio::join!(handle);
        println!("Wakeword loop returned. Restarting in one second.");
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

async fn wakeword_listener_loop(
    wakeword_onnx_file:&str,
    chunk_receiver:&mut mpsc::Receiver<Box<[i16;CHUNK_SIZE]>>,
    last_detected_wakeword_sender:&watch::Sender<Option<SystemTime>>,
    wakeword_threshold:f32
) {
    
    let mut model = match WakeWordModel::new(
        &[wakeword_onnx_file],
        SAMPLE_RATE.try_into().expect("Should convert.")
    ){
        Ok(model)=>model,
        Err(e)=>{
            eprintln!("{:?}",e);
            return;
        }
    };

    loop{
        match chunk_receiver.recv().await
        {
            Some(chunk) => {
                //println!("Received value change.");
                match model.predict(&*chunk)
                {
                    Ok(res) => {
                        for (wakeword, score) in res
                        {
                            if score>wakeword_threshold
                            {
                                match last_detected_wakeword_sender.send(Some(SystemTime::now()))
                                {
                                    Ok(_)=>(),
                                    Err(e)=>{eprintln!("{:?}",e)}
                                }
                                println!("Detected {} at {:?} (score {})",wakeword, std::time::Instant::now(), score);
                            }
                            else
                            {
                                //println!("No detection (score {}, needs {})", score, wakeword_threshold)
                            }
                        }
                    },
                    Err(_) => eprintln!("Model error"),
                };
            },
            None => (),
        }
    }
}