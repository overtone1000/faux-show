use std::boxed;

use hyper::{body::Incoming, Method, Request, Response};
use hyper_services::{
    commons::HandlerResult, request_processing::get_request_body_as_string, response_building::{bad_request, box_existing_full, box_existing_response, bytes_to_boxed_body}, service::stateful_service::StatefulHandler
};

use hyper_tungstenite::{tungstenite, HyperWebsocket};
use tungstenite::Message;
use websocket::WebSocketStreamNext;

#[derive(Clone)]
pub struct InternalHTTPService {
    internal_service_static_directory:String,
}

impl InternalHTTPService
{
    pub fn new(initialization_parameters:&crate::InitializationParameters)->InternalHTTPService
    {
        InternalHTTPService { 
            internal_service_static_directory: initialization_parameters.internal_service_static_directory.clone()
        }
    }
}

impl StatefulHandler for InternalHTTPService {
    async fn handle_request(self:Self, request: Request<Incoming>) -> HandlerResult {
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

                return Ok(Response::new(bytes_to_boxed_body("Ok")));
            },
            Method::GET => {

                println!("Received GET for {:?}",parts.uri);

                return hyper_services::response_building::send_file(&self.internal_service_static_directory,parts.uri.path()).await;
            },
            method=>{
                eprintln!("Received unexpected method {:?}",method);
                return Ok(bad_request());
            }
        }
    }
}