pub(crate) mod services;


use std::
    net::{IpAddr, Ipv4Addr}
;

use hyper_services::service::stateful_service::StatefulService;
use hyper_services::service::stateless_service::StatelessService;
use hyper_services::{ConnectionProperties, spawn_server};

use crate::services::external::ExternalService;
use crate::services::internal::http::InternalHTTPService;
use crate::services::internal::ws::InternalWSService;

#[derive(Debug)]
pub struct InitializationParameters
{
    internal_service_static_directory:String,
    internal_http_port:u16,
    internal_ws_port:u16,
    external_port:u16
}

impl InitializationParameters
{
    pub fn new(internal_service_static_directory:&str, internal_http_port:u16, internal_ws_port:u16, external_port:u16)->InitializationParameters
    {
        InitializationParameters { 
            internal_service_static_directory:internal_service_static_directory.to_string(), 
            internal_http_port, 
            internal_ws_port, 
            external_port 
        }
    }
}

pub async fn start_and_run(params:InitializationParameters) {
    loop {
        
        println!("Starting services.");

        //Create event servers
        let internal_http_service = {

            let handler = InternalHTTPService::new(&params);
            let internal_service=StatefulService::create(handler);
            
            spawn_server(
                IpAddr::V4(Ipv4Addr::UNSPECIFIED),
                params.internal_http_port,
                internal_service,
                ConnectionProperties::default()
            )
        };

        let internal_ws_service = {

            let handler = InternalWSService::new(&params);
            let internal_service=StatefulService::create(handler);
            
            spawn_server(
                IpAddr::V4(Ipv4Addr::UNSPECIFIED),
                params.internal_ws_port,
                internal_service,
                ConnectionProperties{
                    with_upgrades:true
                }
            )
        };

        let external_service = {

            let external_service:StatelessService<ExternalService>=StatelessService::create();
            println!("External service created.");

            spawn_server(
                IpAddr::V4(Ipv4Addr::UNSPECIFIED),
                params.external_port,
                external_service,
                ConnectionProperties::default()
            )
        };

        println!("Services running.");

        match tokio::try_join!(internal_http_service, internal_ws_service, external_service)
        {
            Ok(_) => println!("Services closed gracefully."),
            Err(e) => {
                println!("Service Failure");
                println!("{}", e.to_string());
            }
        }
    }
}
