
use hyper::{body::Incoming, Method, Request, Response};
use hyper_services::{
    commons::HandlerResult, request_processing::{Auth, collect_incoming, get_request_body_as_string}, response_building::{bad_request, bytes_to_boxed_body}, service::{stateful_service::StatefulHandler, stateless_service::StatelessHandler}
};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct ExternalService {
    auth:Auth
}

#[derive(Serialize, Deserialize, Debug)]
struct ChangeDashData {index:u32}

#[derive(Serialize, Deserialize, Debug)]
pub enum Command
{
    ChangeDash(ChangeDashData)
}

impl ExternalService {
    pub fn new(auth:&Auth) -> ExternalService
    {
        ExternalService{auth:auth.clone()}
    }

    fn get_validator(&self)->impl Fn(Auth) -> bool{
        let c=self.auth.clone();
        move |auth|{
            c==auth
        }
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
                                    let deserialized: Command = match serde_json::from_str(&value){
                                        Ok(result)=>result,
                                        Err(e)=>{
                                            eprintln!("Couldn't deserialize command. {:?}",e);
                                            return Ok(bad_request());
                                        }
                                    };
                                    
                                    println!("Got command {:?}",deserialized);
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


#[cfg(test)]
mod tests {

    use super::*;

    fn check_serialization(command: &Command) {
        println!("Serialization test:");
        let serialized = serde_json::to_string(command).expect("Should serialize.");
        println!("   {}", serialized);
        let deserialized: Command = serde_json::from_str(&serialized).expect("Should deserialize.");
        println!("   {:?}", deserialized);
    }

    #[test]
    fn serialization() {
        check_serialization(&Command::ChangeDash(ChangeDashData { index: 3 }));
    }
}
