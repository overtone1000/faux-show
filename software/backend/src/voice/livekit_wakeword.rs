use livekit_wakeword::{SAMPLE_RATE, wakeword::WakeWordModel};
use tokio::sync::mpsc;

const THRESHOLD:f32=0.5;

//Livekit wakeword *MUST* be run in release mode or it is very slow. It also uses quite a bit of CPU.

async fn run_wakeword_listener(wakeword_onnx_file:String) {
    let (chunk_transmitter, mut chunk_receiver) = mpsc::channel::<Box<[i16;CHUNK_SIZE]>>(CHUNK_BUFFER_MULTIPLE);
    
    let mut model = WakeWordModel::new(
        &[wakeword_onnx_file],
        SAMPLE_RATE.try_into().expect("Should convert.")
    ).expect("model should start");

    let model_processor_future = tokio::spawn(
        async move {
            loop{
                //println!("Chunk receiver contains {} chunks.",chunk_receiver.len());
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
                                        println!("Detected {} at {:?} (score {})",wakeword, std::time::Instant::now(), score);
                                    }
                                    else {
                                        println!("No detection. (score {})",score);
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
    );

    match tokio::join!(model_processor_future)
    {
        (Ok(_),) => (),
        (Err(e),) => eprintln!("{:?}",e),
    }
}