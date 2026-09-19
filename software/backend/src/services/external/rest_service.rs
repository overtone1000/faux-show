
use hyper::{body::Incoming, Method, Request, Response};
use hyper_services::{
    commons::HandlerResult, request_processing::{Auth, collect_incoming}, response_building::{bad_request, ok, server_side_failure}, service::{stateful_service::StatefulHandler, stateless_service::StatelessHandler}
};

use tokio::sync::mpsc::UnboundedSender;

use crate::{comm::{CommunicationSpoke, external_commands::Command}, device::set_screen_state};


#[derive(Clone)]
pub struct ExternalService {
    auth:Auth,
    kiosk_uid:u64,
    spoke:CommunicationSpoke
}

impl ExternalService {
    pub fn new(auth:&Auth,kiosk_uid:&u64,spoke:CommunicationSpoke) -> ExternalService
    {
        ExternalService{
            auth:auth.clone(),
            kiosk_uid:kiosk_uid.to_owned(),
            spoke
        }
    }

    fn get_validator(&self)->impl Fn(Auth) -> bool{
        let c=self.auth.clone();
        move |auth|{
            c==auth
        }
    }

    fn handle_command(&self, command:Command)->Result<(),()>
    {
        match command
        {
            Command::AutoTab(_) => {
                println!("Passing directly to internal service without modification.");
                self.spoke.send_external_command(command);
            },
            Command::SetScreenState(state) => {
                match set_screen_state(state, &self.kiosk_uid)
                {
                    Ok(_)=>(),
                    Err(e)=>{
                        eprintln!("{:?}",e);
                    }
                }
            },
            Command::PhotoprismKey(_)=>{
                eprintln!("Invalid command.");
            }
            Command::SetVoiceControlState(_) => {
                eprintln!("Invalid command.");
            },
        }

        Ok(())
    }
}

impl StatefulHandler for ExternalService {
    async fn handle_request(self, request: Request<Incoming>) -> HandlerResult {
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
                                    match serde_json::from_str::<Command>(&value){
                                        Ok(command)=>{
                                            println!("Got command {:?}",command);
                                            match self.handle_command(command)
                                            {
                                                Ok(_)=>(),
                                                Err(_)=>{
                                                    return Ok(server_side_failure());
                                                }
                                            }
                                        },
                                        Err(e)=>{
                                            eprintln!("Couldn't deserialize command. {:?}",e);
                                            return Ok(bad_request());
                                        }
                                    };
                                },
                                key=>{
                                    println!("Unexpected key-value pair {}:{}",key,value);
                                }
                            }
                        }
                        
                        Ok(ok())
                    },
                    Method::GET => {

                        println!("Received GET for {:?}",parts.uri);

                        Ok(ok())
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