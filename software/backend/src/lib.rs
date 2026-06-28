pub(crate) mod services;
pub(crate) mod commands;
pub(crate) mod device;
pub(crate) mod voice;
pub mod mqtt;

use std::collections::HashMap;
use std::
    net::{IpAddr, Ipv4Addr}
;

use has_mqtt::component::HomeAssistantDeviceComponent;
use has_mqtt::device::HomeAssistantDeviceConfiguration;
use has_mqtt::mqtt_client::{DEFAULT_DISCOVERY_PREFIX, HASMQTTClient};
use has_mqtt::platform::switch::state::SwitchState;
use hyper_services::request_processing::Auth;
use hyper_services::service::certificates::generate_simple_certificates;
use hyper_services::service::spawn::ConnectionProperties;
use hyper_services::service::stateful_service::StatefulService;

use crate::mqtt::MQTTConfiguration;
use crate::services::external::external_core::ExternalCore;
use crate::services::external::rest_service::ExternalService;
use crate::services::internal::InternalService;

#[derive(Debug)]
pub struct InitializationParameters
{
    internal_service_static_directory:String,
    config_static_directory:String,
    internal_port:u16,
    external_port:u16,
    auth:Auth,
    kiosk_uid:u64,
    mqtt_config:MQTTConfiguration,
    photoprism_key:String
}

impl InitializationParameters
{
    pub fn new(
        internal_service_static_directory:&str,
        config_static_directory:&str,
        internal_port:u16,
        external_port:u16,
        auth:Auth,
        kiosk_uid:u64,
        mqtt_config:MQTTConfiguration,
        photoprism_key:String
    )->InitializationParameters
    {
        InitializationParameters { 
            internal_service_static_directory:internal_service_static_directory.to_string(),
            config_static_directory:config_static_directory.to_string(),
            internal_port, 
            external_port,
            auth,
            kiosk_uid,
            mqtt_config,
            photoprism_key
        }
    }
}

pub async fn start_and_run(params:InitializationParameters) {
    loop {
        
        println!("Starting services.");

        //Command receiver doesn't implement clone so can't pass it in to a service.
        let (command_sender, command_receiver) = tokio::sync::mpsc::unbounded_channel::<commands::Command>();

        let external_core=ExternalCore::new(command_sender);

        let internal_handler = InternalService::new(&params, std::sync::Arc::new(tokio::sync::Mutex::new(command_receiver)));
        let external_handler = ExternalService::new(&params.auth,&params.kiosk_uid,external_core.clone());

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


        let mqtt_client=mqtt::get_has_client(external_core, &params.mqtt_config, params.kiosk_uid).await;
        let mqtt_client_future = mqtt_client.run();

        println!("Services created.");

        match tokio::try_join!(internal_service_future, external_service_future, mqtt_client_future)
        {
            Ok(_) => println!("Services closed gracefully."),
            Err(e) => {
                println!("Service Failure");
                println!("{}", e.to_string());
            }
        }
    }
}
