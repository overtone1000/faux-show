use std::{collections::{HashMap, HashSet, VecDeque}, sync::Arc, thread::current, time::{Duration, Instant}};

use futures_util::stream::{SplitSink, SplitStream};
use hyper::{Method, Request, Response, body::Incoming, upgrade::Upgraded};
use hyper_services::{
    commons::HandlerResult, request_processing::get_request_body_as_string, response_building::{bad_request, box_existing_full, box_existing_response, bytes_to_boxed_body}, service::stateful_service::StatefulHandler
};

use hyper_tungstenite::{HyperWebsocket, WebSocketStream, tungstenite};
use hyper_util::rt::TokioIo;
use tokio::sync::{Mutex, mpsc::{self, UnboundedReceiver, UnboundedSender}};
use tungstenite::Message;
use websocket::WebSocketStreamNext;

use futures_util::SinkExt;
use futures_util::StreamExt;

use crate::commands::Command;

const CONFIG_PREFACE:&str="/config";

#[derive(Clone)]
pub struct InternalService {
    internal_service_static_directory:String,
    config_static_directory:String,
    command_receiver:Arc<Mutex<UnboundedReceiver<Command>>>,
    sinks:Arc<Mutex<HashMap<u64,SplitSink<WebSocketStream<TokioIo<Upgraded>>,Message>>>>,
    sink_handler:Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>
}

impl InternalService
{
    pub fn new(initialization_parameters:&crate::InitializationParameters, command_receiver:Arc<Mutex<UnboundedReceiver<Command>>>)->InternalService
    {        
        InternalService { 
            internal_service_static_directory: initialization_parameters.internal_service_static_directory.clone(),
            config_static_directory: initialization_parameters.config_static_directory.clone(),
            command_receiver,
            sinks:Arc::new(Mutex::new(HashMap::new())),
            sink_handler:Arc::new(Mutex::new(None))
        }
    }

    //commands:Arc<Mutex<VecDeque<Command>>>
    async fn handle_websocket_sink(command_receiver:Arc<Mutex<UnboundedReceiver<Command>>>, mut sink:Arc<Mutex<HashMap<u64,SplitSink<WebSocketStream<TokioIo<Upgraded>>,Message>>>>,)->()
    {
        println!("Starting websocket sink handler.");
        let mut command_receiver= command_receiver.lock().await;

        loop {
            match command_receiver.recv().await
            {
                Some(command)=>{
                    match serde_json::to_string(&command)
                    {
                        Ok(command_as_string)=>{
                            let mut locked_sinks = sink.lock().await;
                            for (_key,sink) in &mut *locked_sinks
                            {
                                match sink.send(Message::text(command_as_string.clone())).await
                                {
                                    Ok(_)=>{
                                        println!("Command sent via websocket.");
                                    },
                                    Err(e)=>{
                                        eprintln!("Websocket send error: {:?}",e);
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Couldn't deserialize command. {:?}",e);
                        },
                    }
                },
                None=>{
                    //stream has closed, exit
                    println!("Command receiver has closed. Closing sink handler.");
                    return;
                }
            }
        }
    }

    async fn handle_websocket_stream(mut stream:SplitStream<WebSocketStream<TokioIo<Upgraded>>>)->()
    {
        println!("Starting websocket stream handler.");
        
        loop {
            match stream.next().await
            {
                Some(stream_next)=>{
                    match stream_next {
                        Ok(stream_next)=>{
                            match stream_next {
                                Message::Text(msg) => {
                                    //Don't really do anything with messages from the client yet.
                                    println!("Received text message: {msg}");
                                },
                                Message::Ping(_)=>println!("Ping"),
                                Message::Pong(_)=>println!("Pong"),
                                _=>() //Ignore all other message types.
                            }
                        },
                        Err(e) => {
                            eprintln!("Websocket error: {:?}",e);
                            return
                        },
                    }
                }
                None=>{
                    //stream is done, exit
                    println!("Stream has closed. Closing stream handler.");
                    return 
                }
            }            
        }
    }

    async fn handle_websocket(self, websocket: HyperWebsocket) -> () {       

        println!("Serving websocket");
        let websocketstream = match websocket.await{
            Ok(websocketstream) => websocketstream,
            Err(e) => {
                eprintln!("Websocket error: {:?}",e);
                return;
            },
        };

        let (sink,stream) =websocketstream.split();
        let command_receiver = self.command_receiver;        

        //Add the sink to the sink vector. Make sure a sink handler is running. If it is, let it continue.
        let sink_key={
            let mut sinks = self.sinks.lock().await;
            let mut current_handler = self.sink_handler.lock().await;

            let mut sink_key=0u64;
            while sinks.contains_key(&sink_key)
            {
                sink_key+=1;
            }
            sinks.insert(sink_key,sink);

            match *current_handler
            {
                Some(_) => (),
                None => {
                    let sinksclone=self.sinks.clone();
                    *current_handler=Some(tokio::spawn(async move {Self::handle_websocket_sink(command_receiver, sinksclone).await}))
                },
            };

            sink_key
        };

       match tokio::spawn(async move {Self::handle_websocket_stream(stream).await}).await
       {
            Ok(_)=>(),
            Err(e) => {
                eprintln!("Websocket error: {:?}",e);
                return;
            },
       }

       println!("Closed websocket. Cleaning up.");

       //Remove the sink from the sink vec
       {
            let mut sinks = self.sinks.lock().await;
            let mut current_handler = self.sink_handler.lock().await;

            sinks.remove(&sink_key);

            if sinks.len()<=0
            {
                match &*current_handler
                {
                    Some(handler) => handler.abort(),
                    None => ()
                }
            }

            *current_handler=None
        }

       //If there are no more sinks, stop the sink handler
    }
}

impl StatefulHandler for InternalService {
    async fn handle_request(self:Self, request: Request<Incoming>) -> HandlerResult {

        match hyper_tungstenite::is_upgrade_request(&request) {
            true=>{
                let (response, websocket) = hyper_tungstenite::upgrade(request, None)?;
            
                println!("Received websocket request. Response is {:?}", response);
                // Spawn a task to handle the websocket connection.
                tokio::spawn(async move {
                    self.handle_websocket(websocket).await
                });

                // Return the response so the spawned future can continue.
                let boxed_response=box_existing_response(response);
                println!("Boxed response is {:?}", boxed_response);
                Ok(boxed_response)
            },
            false=>{
                let (parts, incoming) = request.into_parts();
                        
                match parts.method {
                    Method::POST => {
                        let body= match get_request_body_as_string(incoming).await
                        {
                            Ok(body)=>body,
                            Err(e)=>{
                                eprintln!("Couldn't get request body. {:?}",e);
                                return Ok(bad_request());
                            }
                        };

                        println!("Received POST {:?} with body {:?}",parts.uri, body);

                        Ok(Response::new(bytes_to_boxed_body("Ok")))
                    },
                    Method::GET => {
                        //println!("Received GET for {:?}",parts.uri);                       
                        
                        if parts.uri.path().starts_with(CONFIG_PREFACE){
                            let final_path=parts.uri.path().split_at(CONFIG_PREFACE.len()).1;
                            //println!("Serving config {:?} - {:?}",&self.config_static_directory,final_path);
                            hyper_services::response_building::send_file(&self.config_static_directory,final_path).await
                        }
                        else {
                            //println!("Serving base.");
                            hyper_services::response_building::send_file(&self.internal_service_static_directory,parts.uri.path()).await
                        }
                    },
                    method=>{
                        eprintln!("Received unexpected method {:?}",method);
                        Ok(bad_request())
                    }
                }
            }   
        }
    }
}


