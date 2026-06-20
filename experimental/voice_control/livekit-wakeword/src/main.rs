use cpal::{StreamConfig, traits::{DeviceTrait, HostTrait, StreamTrait}};
use livekit_wakeword::wakeword::WakeWordModel;

const SAMPLE_RATE:usize=16000;
const CHUNK_TIME_MILLISECONDS:usize=2000;
const CHANNELS:u16=1;
const BITS_PER_SAMPLE:u32=16;
const CHUNK_SIZE:usize=SAMPLE_RATE*CHUNK_TIME_MILLISECONDS;


#[tokio::main]
async fn main() {

    let host = cpal::default_host();
    let device = host.default_input_device().expect("Should exist.");

    let sample_rate_u32=SAMPLE_RATE.try_into().expect("Should convert.");

    for supported_config in device.supported_input_configs().expect("Should get configs")
    {
        if supported_config.sample_format().is_int() && 
            supported_config.sample_format().bits_per_sample()==BITS_PER_SAMPLE &&
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

    let mut model = WakeWordModel::new(&["/home/tyler/repos/faux-show/experimental/voice_control/livekit-wakeword/hey_livekit.onnx"], SAMPLE_RATE.try_into().expect("Should convert.")).expect("model should start");

    let raw_data_buffer_index:usize=0;
    let raw_data_buffer:[i16;CHUNK_SIZE];
    let data_fn = move |data: &[i16], _: &cpal::InputCallbackInfo| {
    };

    let stream = device.build_input_stream(
        stream_config,
        data_fn,
        err_fn,
        None, // Timeout
    ).expect("Failed to build input stream");

    stream.play().expect("Failed to start stream");

    // Feed ~2s PCM audio chunks (i16, at configured sample rate)
    //let scores = model.predict(&audio_chunk)?;
    //if scores["hey_livekit"] > 0.5 {
    //    println!("Wake word detected!");
    //}

}