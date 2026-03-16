
use hyper::{body::Incoming, Method, Request, Response};
use hyper_services::{
    commons::HandlerResult, request_processing::{Auth, collect_incoming}, response_building::{bad_request, bytes_to_boxed_body}, service::{stateful_service::StatefulHandler, stateless_service::StatelessHandler}
};
use tokio::sync::mpsc::UnboundedSender;

use crate::{commands::Command};

#[derive(Clone)]
pub struct ExternalService {
    auth:Auth,
    //internal_handler:InternalService
    command_sender:UnboundedSender<Command>
}

impl ExternalService {
    pub fn new(auth:&Auth,command_sender:UnboundedSender<Command>) -> ExternalService
    {
        ExternalService{
            auth:auth.clone(),
            command_sender
        }
    }

    fn get_validator(&self)->impl Fn(Auth) -> bool{
        let c=self.auth.clone();
        move |auth|{
            c==auth
        }
    }
}

impl StatefulHandler for ExternalService {
    async fn handle_request(mut self, request: Request<Incoming>) -> HandlerResult {
        let (parts, incoming) = request.into_parts();

        match hyper_services::request_processing::check_basic_authentication(&parts,"/",self.get_validator()).await
        {
            hyper_services::commons::Handler::Continue => {
                println!("Authenticated.");
                match parts.method {
                    Method::POST => {

                        let collected = collect_incoming(incoming).await?.to_bytes().to_vec(); 

                        let decoded = form_urlencoded::parse(&collected);

                        for (key,value) in decoded
                        {
                            match key
                            {
                                std::borrow::Cow::Borrowed("message")=>{
                                    println!("Decoding command.");
                                    let deserialized: Command = match serde_json::from_str(&value){
                                        Ok(result)=>result,
                                        Err(e)=>{
                                            eprintln!("Couldn't deserialize command. {:?}",e);
                                            return Ok(bad_request());
                                        }
                                    };
                                    
                                    println!("Got command {:?}, passing to internal service",deserialized);
                                    match self.command_sender.send(deserialized)
                                    {
                                        Ok(_)=>(),
                                        Err(e)=>{
                                            eprintln!("Error during internal command processing. {:?}",e);
                                            return Ok(bad_request());
                                        }
                                    }
                                },
                                key=>{
                                    println!("Unexpected key-value pair {}:{}",key,value);
                                }
                            }
                        }
                        
                        Ok(Response::new(bytes_to_boxed_body("Ok")))
                    },
                    Method::GET => {

                        println!("Received GET for {:?}",parts.uri);

                        Ok(Response::new(bytes_to_boxed_body("Ok")))
                    },
                    method=>{
                        eprintln!("Received unexpected method {:?}",method);
                        Ok(bad_request())
                    }
                }   
            },
            hyper_services::commons::Handler::ImmediateReturn(response) => Ok(response),
            hyper_services::commons::Handler::Error(error) => Err(error),
        }
    }
}