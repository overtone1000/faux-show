use std::time::Duration;

use tokio_tungstenite::connect_async;
use futures_util::{SinkExt, StreamExt};

#[tokio::main]
async fn main() {
    let url = "ws://127.0.0.1:9090";

    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
    println!("WebSocket handshake has been successfully completed");

    let (mut write, mut read) = ws_stream.split();

    let test_send = "{\"uid\": \"test_client\",\"language\": \"en\",\"model\": \"base\",\"use_vad\": false}";

    println!("{}",test_send);

    //write.send(test_send.into()).await.expect("Should complete.");
    
    //Seems should just feed and not "send" or "flush" otherwise websocket gets closed...but this doesn't seem to initialize the server.
    println!("Waiting...");
    std::thread::sleep(Duration::from_secs(3));
    println!("Sending...");
    write.feed(test_send.into()).await.expect("Should complete.");
    println!("Sent.");

    loop {
        match read.next().await
        {
            Some(next) => {
                match next
                {
                    Ok(next)=>{ 
                        println!("Websocket received: {:?}",next);
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