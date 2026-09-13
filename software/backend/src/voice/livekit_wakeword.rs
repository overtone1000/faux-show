use std::time::SystemTime;

use livekit_wakeword::{SAMPLE_RATE, wakeword::WakeWordModel};

use tokio::sync::{mpsc, watch};

use crate::voice::audio_stream::CHUNK_SIZE;

const THRESHOLD:f32=0.5;

//Livekit wakeword *MUST* be run in release mode or it is very slow. It also uses quite a bit of CPU.

//Make chunk sender and chunk receiver pair with
//let (chunk_transmitter, mut chunk_receiver) = mpsc::channel::<Box<[i16;CHUNK_SIZE]>>(CHUNK_BUFFER_MULTIPLE);

pub async fn run_wakeword_listener(
    wakeword_onnx_file:String,
    mut chunk_receiver:mpsc::Receiver<Box<[i16;CHUNK_SIZE]>>,
    last_detected_wakeword_sender:watch::Sender<Option<SystemTime>>
)
{
    loop {
        let handle=wakeword_listener_loop(
            &wakeword_onnx_file, 
            &mut chunk_receiver,
            &last_detected_wakeword_sender
        );
        tokio::join!(handle);
    }
}

async fn wakeword_listener_loop(
    wakeword_onnx_file:&str,
    chunk_receiver:&mut mpsc::Receiver<Box<[i16;CHUNK_SIZE]>>,
    last_detected_wakeword_sender:&watch::Sender<Option<SystemTime>>
) {
    
    let mut model = WakeWordModel::new(
        &[wakeword_onnx_file],
        SAMPLE_RATE.try_into().expect("Should convert.")
    ).expect("model should start");

    let model_processor_function = async move {
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
                                if score>THRESHOLD
                                {
                                    match last_detected_wakeword_sender.send(Some(SystemTime::now()))
                                    {
                                        Ok(_)=>(),
                                        Err(e)=>{eprintln!("{:?}",e)}
                                    }
                                    println!("Detected {} at {:?} (score {})",wakeword, std::time::Instant::now(), score);
                                }
                            }
                        },
                        Err(_) => eprintln!("Model error"),
                    };
                },
                None => (),
            }
        }
    };

    //let model_processor_future = tokio::spawn(model_processor_function);

    tokio::select!{
        _=model_processor_function=>{eprintln!("Model processor function exited.");}
    }
}