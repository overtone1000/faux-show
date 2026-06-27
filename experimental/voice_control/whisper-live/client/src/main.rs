use std::time::Duration;

use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};

#[tokio::main]
async fn main() {
    let url = "ws://127.0.0.1:9090";

    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
    println!("WebSocket handshake has been successfully completed");

    let (mut write, mut read) = ws_stream.split();

    let test_send = "{\"uid\": \"test_client\",\"language\": \"en\",\"model\": \"base\",\"use_vad\": false,\"task\":\"live-stt\"}";

    println!("{}",test_send);

    //write.send(test_send.into()).await.expect("Should complete.");

    let test_message = Message::text(test_send);
    
    write.send(test_message).await.expect("Should complete.");

    loop {
        match read.next().await
        {
            Some(next) => {
                match next
                {
                    Ok(next)=>{ 
                        println!("Websocket received: {:?}",next);
                        match next
                        {
                            Message::Text(_utf8_bytes) => (),
                            Message::Binary(_bytes) => (),
                            Message::Ping(_bytes) => (),
                            Message::Pong(_bytes) => (),
                            Message::Close(close_frame) => {
                                match close_frame
                                {
                                    Some(_close_frame)=>(),
                                    None=>()
                                }
                            },
                            Message::Frame(_frame) => (),
                        }
                    },
                    Err(e)=>{
                        eprintln!("{:?}",e);
                    }
                }
            },
            None => (),
        }
    }

    
    //std::thread::sleep(Duration::from_secs(10));
}