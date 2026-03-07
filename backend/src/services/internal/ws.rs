
use std::boxed;

use hyper::{body::Incoming, Method, Request, Response};
use hyper_services::{
    commons::HandlerResult, request_processing::get_request_body_as_string, response_building::{bad_request, box_existing_full, box_existing_response, bytes_to_boxed_body}, service::stateful_service::StatefulHandler
};

use hyper_tungstenite::{tungstenite, HyperWebsocket};
use tungstenite::Message;
use websocket::WebSocketStreamNext;

#[derive(Clone)]
pub struct InternalWSService {
}

impl InternalWSService
{
    pub fn new()->InternalWSService
    {
        InternalWSService { 
        }
    }
}

impl StatefulHandler for InternalWSService {
    async fn handle_request(self:Self, request: Request<Incoming>) -> HandlerResult {

        if hyper_tungstenite::is_upgrade_request(&request) {
            let (response, websocket) = hyper_tungstenite::upgrade(request, None)?;
            
            println!("Received websocket request. Response is {:?}", response);
            // Spawn a task to handle the websocket connection.
            tokio::spawn(async move {
                serve_websocket(websocket).await
            });

            // Return the response so the spawned future can continue.
            let boxed_response=box_existing_response(response);
            println!("Boxed response is {:?}", boxed_response);
            return Ok(boxed_response);
        } else {
            println!("Unexpected traffic in web socket service.");
            return Ok(bad_request());
        }
    }
}

/// Handle a websocket connection.
async fn serve_websocket(websocket: HyperWebsocket) -> () {
    let send_response = async |mut stream_next:WebSocketStreamNext, message:Message| {
        match stream_next.send_message(message).await
        {
            Ok(_)=>(),
            Err(e)=>eprintln!("{:?}",e)
        };
    };
    
    println!("Serving websocket");
    match WebSocketStreamNext::get_next(websocket).await
    {
        Ok(stream_next) => {           
            println!("Got next.");
            match stream_next.get_message() {
                Message::Text(msg) => {
                    println!("Received text message: {msg}");
                    send_response(stream_next, Message::text("Thank you, come again.")).await;
                },
                Message::Binary(msg) => {
                    println!("Received binary message: {msg:02X?}");
                    send_response(stream_next, Message::binary(b"Thank you, come again.".to_vec())).await;
                },
                Message::Ping(msg) => {
                    // No need to send a reply: tungstenite takes care of this for you.
                    println!("Received ping message: {msg:02X?}");
                },
                Message::Pong(msg) => {
                    println!("Received pong message: {msg:02X?}");
                }
                Message::Close(msg) => {
                    // No need to send a reply: tungstenite takes care of this for you.
                    if let Some(msg) = &msg {
                        println!("Received close message with code {} and message: {}", msg.code, msg.reason);
                    } else {
                        println!("Received close message");
                    }
                },
                Message::Frame(_msg) => {
                    println!("Unanticipated Frame.");
                    unreachable!();
                }
            }            
        },
        Err(e) => {
            eprintln!("Websocket error: {:?}",e);
        },
    }
}
