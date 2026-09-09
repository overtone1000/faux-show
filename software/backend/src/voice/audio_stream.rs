use cpal::{StreamConfig, traits::{DeviceTrait, HostTrait, StreamTrait}};

const SAMPLE_RATE:usize=16000;
const CHANNELS:u16=1;
const BITS_PER_SAMPLE:usize=16;
const CHUNK_TIME_MILLISECONDS:usize=2000;
const CHUNK_SIZE:usize=SAMPLE_RATE*CHUNK_TIME_MILLISECONDS/1000;
const AUDIO_STEP_SIZE:usize=CHUNK_SIZE/2;
const AUDIO_BUFFER_MULTIPLE:usize=2; //risk of stack overflow here, keep this low
const AUDIO_BUFFER_SIZE:usize=AUDIO_BUFFER_MULTIPLE*CHUNK_SIZE;
const CHUNK_BUFFER_MULTIPLE:usize=5;

/* Whisper data function
    
    let data_fn = move |data: &[i16], _: &cpal::InputCallbackInfo| {
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

        websocket_message_transmitter.blocking_send(Message::binary(
                Bytes::from_iter(raw_bytes)
        )).expect("Shoud send.");
    };
*/

/* Livekit data function
    
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

 */

fn build_stream()
{
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

    let err_fn = move |err| {
        eprintln!("An error occurred on the audio stream: {}", err);
    };

    let stream = device.build_input_stream(
        stream_config,
        data_fn,
        err_fn,
        None, // Timeout
    ).expect("Failed to build input stream");

    stream.play().expect("Failed to start stream");
}