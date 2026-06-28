use circular_buffer::CircularBuffer;
use cpal::{StreamConfig, traits::{DeviceTrait, HostTrait, StreamTrait}};
use livekit_wakeword::wakeword::WakeWordModel;
use tokio::sync::mpsc;

const SAMPLE_RATE:usize=16000;
const CHUNK_TIME_MILLISECONDS:usize=2000;
const CHANNELS:u16=1;
const BITS_PER_SAMPLE:usize=16;
const CHUNK_SIZE:usize=SAMPLE_RATE*CHUNK_TIME_MILLISECONDS/1000;
const AUDIO_STEP_SIZE:usize=CHUNK_SIZE/2;
const AUDIO_BUFFER_MULTIPLE:usize=2; //risk of stack overflow here, keep this low
const AUDIO_BUFFER_SIZE:usize=AUDIO_BUFFER_MULTIPLE*CHUNK_SIZE;
const CHUNK_BUFFER_MULTIPLE:usize=5;

const THRESHOLD:f32=0.5;
const HEY_LIVEKIT:&str="hey_livekit";

//Livekit wakeword *MUST* be run in release mode or it is very slow. It also uses quite a bit of CPU.

#[tokio::main]
async fn main() {

    let bits_per_sample_u32:u32 = BITS_PER_SAMPLE.try_into().expect("Should convert.");
    let sample_rate_u32:u32 = SAMPLE_RATE.try_into().expect("Should convert.");

    let (chunk_transmitter, mut chunk_receiver) = mpsc::channel::<Box<[i16;CHUNK_SIZE]>>(CHUNK_BUFFER_MULTIPLE);

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

    let mut model = WakeWordModel::new(
        &["/home/tyler/repos/faux-show/experimental/voice_control/livekit-wakeword/".to_string() + HEY_LIVEKIT + ".onnx"],
        SAMPLE_RATE.try_into().expect("Should convert."
    )).expect("model should start");

    println!("Chunk size is {:?}",CHUNK_SIZE);
    println!("Buffer size is {:?}",AUDIO_BUFFER_SIZE);
    let mut audio_buffer = CircularBuffer::<AUDIO_BUFFER_SIZE,i16>::new();
    let data_fn = move |data: &[i16], _: &cpal::InputCallbackInfo| {
        //Move data to buffer
        audio_buffer.extend_from_slice(data);

        //println!("Got {} data, {} currently in buffer.",data.len(),audio_buffer.len());
        //Send data to model if enough is buffered
        while audio_buffer.len()>CHUNK_SIZE
        {
            let mut chunk:Box<[i16;CHUNK_SIZE]>=Box::new([0;CHUNK_SIZE]);
            for n in 0..CHUNK_SIZE
            {
                chunk[n]=*audio_buffer.nth_front(n).expect("Should exist");
            }
            
            match chunk_transmitter.blocking_send(chunk)
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
    };

    let stream = device.build_input_stream(
        stream_config,
        data_fn,
        err_fn,
        None, // Timeout
    ).expect("Failed to build input stream");

    stream.play().expect("Failed to start stream");

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
                                        println!("Detected {} at {:?} (score {})",wakeword, std::time::Instant::now(), score)
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