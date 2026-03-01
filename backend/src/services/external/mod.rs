
use hyper::{body::Incoming, Method, Request, Response};
use hyper_services::{
    commons::HandlerResult, request_processing::get_request_body_as_string, response_building::{bad_request, full_to_boxed_body}, service::{stateless_service::StatelessHandler}
};
#[derive(Clone)]
pub struct ExternalService {
}

impl StatelessHandler for ExternalService {
    async fn handle_request(request: Request<Incoming>) -> HandlerResult {
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

                return Ok(Response::new(full_to_boxed_body("Ok")));
            },
            Method::GET => {

                println!("Received GET for {:?}",parts.uri);

                return Ok(Response::new(full_to_boxed_body("Ok")));
            },
            method=>{
                eprintln!("Received unexpected method {:?}",method);
                return Ok(bad_request());
            }
        }   
    }
}