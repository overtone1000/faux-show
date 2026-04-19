pub(crate) mod services;
pub(crate) mod commands;


use std::
    net::{IpAddr, Ipv4Addr}
;

use hyper_services::request_processing::Auth;
use hyper_services::service::certificates::generate_simple_certificates;
use hyper_services::service::spawn::ConnectionProperties;
use hyper_services::service::stateful_service::StatefulService;

use crate::services::external::ExternalService;
use crate::services::internal::InternalService;

#[derive(Debug)]
pub struct InitializationParameters
{
    internal_service_static_directory:String,
    config_static_directory:String,
    internal_port:u16,
    external_port:u16,
    auth:Auth
}

impl InitializationParameters
{
    pub fn new(internal_service_static_directory:&str, config_static_directory:&str, internal_port:u16, external_port:u16, auth:Auth)->InitializationParameters
    {
        InitializationParameters { 
            internal_service_static_directory:internal_service_static_directory.to_string(),
            config_static_directory:config_static_directory.to_string(),
            internal_port, 
            external_port,
            auth
        }
    }
}

pub async fn start_and_run(params:InitializationParameters) {
    loop {
        
        println!("Starting services.");

        let (command_sender, command_receiver) = tokio::sync::mpsc::unbounded_channel::<commands::Command>();

        let internal_handler = InternalService::new(&params, std::sync::Arc::new(tokio::sync::Mutex::new(command_receiver)));
        let external_handler = ExternalService::new(&params.auth,command_sender);

        let internal_service= StatefulService::create(internal_handler);
        let external_service = StatefulService::create(external_handler);

        let internal_service_future = internal_service.start(
            IpAddr::V4(Ipv4Addr::LOCALHOST),
            params.internal_port,
            ConnectionProperties{
                with_upgrades:true,
                tls:None
            }
        );

        let external_service_future = match generate_simple_certificates(["*".to_string()])
        {
            Ok(keypair)=>{
                
                external_service.start(
                    IpAddr::V4(Ipv4Addr::UNSPECIFIED),
                    params.external_port,
                    ConnectionProperties{
                        with_upgrades:false,
                        tls:Some(keypair)
                    }
                )
            },
            Err(e)=>{
                panic!("Couldn't create certificates. {:?}",e);
            }
        };

        println!("Services created.");

        match tokio::try_join!(internal_service_future, external_service_future)
        {
            Ok(_) => println!("Services closed gracefully."),
            Err(e) => {
                println!("Service Failure");
                println!("{}", e.to_string());
            }
        }
    }
}
