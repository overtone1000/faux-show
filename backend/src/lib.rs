pub(crate) mod services;


use std::
    net::{IpAddr, Ipv4Addr}
;

use hyper_services::service::stateful_service::StatefulService;
use hyper_services::service::stateless_service::StatelessService;
use hyper_services::{ConnectionProperties};

use crate::services::external::ExternalService;
use crate::services::internal::InternalService;

#[derive(Debug)]
pub struct InitializationParameters
{
    internal_service_static_directory:String,
    config_static_directory:String,
    internal_port:u16,
    external_port:u16
}

impl InitializationParameters
{
    pub fn new(internal_service_static_directory:&str, config_static_directory:&str, internal_port:u16, external_port:u16)->InitializationParameters
    {
        InitializationParameters { 
            internal_service_static_directory:internal_service_static_directory.to_string(),
            config_static_directory:config_static_directory.to_string(),
            internal_port, 
            external_port 
        }
    }
}

pub async fn start_and_run(params:InitializationParameters) {
    loop {
        
        println!("Starting services.");

        //Create event servers
        let internal_service = {

            let handler = InternalService::new(&params);
            let service=StatefulService::create(handler);
            
            service.start(
                IpAddr::V4(Ipv4Addr::UNSPECIFIED),
                params.internal_port,
                ConnectionProperties{
                    with_upgrades:true
                }
            )
        };

        let external_service = {

            let service:StatelessService<ExternalService>=StatelessService::create();

            service.start(
                IpAddr::V4(Ipv4Addr::UNSPECIFIED),
                params.external_port,
                ConnectionProperties::default()
            )
        };

        println!("Services created.");

        match tokio::try_join!(internal_service, external_service)
        {
            Ok(_) => println!("Services closed gracefully."),
            Err(e) => {
                println!("Service Failure");
                println!("{}", e.to_string());
            }
        }
    }
}
