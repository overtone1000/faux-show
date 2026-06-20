use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use livekit_wakeword::wakeword::WakeWordModel;

const SAMPLE_RATE:usize=16000;
const CHUNK_TIME_MILLISECONDS:usize=2000;
const CHUNK_SIZE:usize=SAMPLE_RATE*CHUNK_TIME_MILLISECONDS;


#[tokio::main]
async fn main() {

    let host = cpal::default_host();
    let device = host.default_input_device().expect("Should exist.");

    for config in device.supported_input_configs().iter_mut()
    {
        println!("Config: {:?}", config);
    }

    let config = device.default_input_config().expect("Should have config.");
    println!("Default input config: {:?}", config);

    let err_fn = move |err| {
        eprintln!("An error occurred on the audio stream: {}", err);
    };

    let stream_config = config.config();
    let raw_data_buffer:[f32;CHUNK_SIZE];
    let data_fn = move |data: &[f32], _: &cpal::InputCallbackInfo| {
        // Here, 'data' is a slice of floating-point audio samples
        // You can process them in real-time or send them over an async channel
        println!("Captured {} audio samples", data.len());
    };

    let stream = device.build_input_stream(
        stream_config,
        data_fn,
        err_fn,
        None, // Timeout
    ).expect("Failed to build input stream");

    stream.play().expect("Failed to start stream");

    std::thread::sleep(std::time::Duration::from_secs(10));
    
    let mut model = WakeWordModel::new(&["/home/tyler/repos/faux-show/experimental/voice_control/livekit-wakeword/hey_livekit.onnx"], SAMPLE_RATE.try_into().expect("Should convert.")).expect("model should start");

    // Feed ~2s PCM audio chunks (i16, at configured sample rate)
    //let scores = model.predict(&audio_chunk)?;
    //if scores["hey_livekit"] > 0.5 {
    //    println!("Wake word detected!");
    //}

}