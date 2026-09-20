pub(crate) mod services;
pub(crate) mod comm;
pub(crate) mod device;
pub(crate) mod voice;

pub mod mqtt;

use std::
    net::{IpAddr, Ipv4Addr}
;

use hyper_services::request_processing::Auth;
use hyper_services::service::certificates::generate_simple_certificates;
use hyper_services::service::spawn::ConnectionProperties;
use hyper_services::service::stateful_service::StatefulService;

use crate::comm::CommunicationHub;
use crate::mqtt::MQTTConfiguration;
use crate::services::external::rest_service::ExternalService;
use crate::services::internal::InternalService;
use crate::voice::audio_stream::{run_voice_command_listener};

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
    photoprism_key:String,
    whisper_server_url:String,
    wakeword_onnx_file:String
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
        photoprism_key:String,
        whisper_server_url:String,
        wakeword_onnx_file:String
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
            photoprism_key,
            whisper_server_url,
            wakeword_onnx_file
        }
    }
}

pub async fn start_and_run(params:InitializationParameters) {
    loop {
        
        println!("Starting services.");

        //Command receiver doesn't implement clone so can't pass it in to a service.
        //let (external_command_sender, external_command_receiver) = tokio::sync::mpsc::unbounded_channel::<Command>();

        let (mut hub, external_command_receiver)=CommunicationHub::new();
        let spoke=hub.spoke().clone();
        //let (internal_service_notification_sender, internal_service_notification_receiver) = tokio::sync::broadcast::channel::<InternalServiceNotification>(10);
        
        let external_handler = ExternalService::new(&params.auth,&params.kiosk_uid,hub.spoke().clone());
        let external_service = StatefulService::create(external_handler);

        let internal_handler = InternalService::new(
            &params,
            std::sync::Arc::new(tokio::sync::Mutex::new(external_command_receiver)), //partial move of hub
            spoke.clone()
            //internal_service_notification_sender.clone()
        );
        let internal_service= StatefulService::create(internal_handler);  

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


        let mqtt_client=mqtt::get_has_client(spoke.clone(), &params.mqtt_config, params.kiosk_uid).await;
        let mqtt_client_future = mqtt_client.run();

        let voice_command_handle=run_voice_command_listener(
            &params,
            spoke
        );

        let hub_future=hub.start();

        println!("Services created.");

        match tokio::try_join!(
            hub_future,
            internal_service_future,
            external_service_future,
            mqtt_client_future,
            voice_command_handle
        )
        {
            Ok(_) => println!("Services closed gracefully."),
            Err(e) => {
                println!("Service Failure");
                println!("{}", e.to_string());
            }
        }
    }
}
