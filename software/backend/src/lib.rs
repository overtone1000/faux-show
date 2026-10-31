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

use crate::comm::{CommunicationHub, CommunicationSpoke};
use crate::mqtt::MQTTConfiguration;
use crate::services::external::rest_service::ExternalService;
use crate::services::internal::InternalService;
use crate::voice::audio_stream::{WakewordWhisperState, spawn_voice_command_listener};

#[derive(Debug, Clone)]
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
    has_key:String,
    ha_url:String,
    whisper_server_url:String,
    wakeword_onnx_file:String
}

const _: () = {
    const fn assert_send<T: Send>() {}
    assert_send::<InitializationParameters>();
    assert_send::<WakewordWhisperState>();
    assert_send::<comm::CommunicationSpoke>();
    assert_send::<has_mqtt::mqtt_client::HASMQTTClient>();
    assert_send::<&has_mqtt::mqtt_client::HASMQTTClient>();
    assert_send::<has_mqtt::component::HomeAssistantDeviceComponent>();
    assert_send::<rumqttc::AsyncClient>();
    assert_send::<CommunicationSpoke>();
    assert_send::<CommunicationHub>();
    assert_send::<tokio::sync::watch::Sender<WakewordWhisperState>>();
};

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
        has_key:String,
        ha_url:String,
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
            has_key,
            ha_url,
            whisper_server_url,
            wakeword_onnx_file
        }
    }
}

pub async fn start_and_run(params:InitializationParameters) {

    loop {
        
        println!("Starting faux-show main loop.");

        //Command receiver doesn't implement clone so can't pass it in to a service.
        //let (external_command_sender, external_command_receiver) = tokio::sync::mpsc::unbounded_channel::<Command>();

        let (mut hub, external_command_receiver)=CommunicationHub::new();
        let spoke=hub.spoke().clone();
        //let (internal_service_notification_sender, internal_service_notification_receiver) = tokio::sync::broadcast::channel::<InternalServiceNotification>(10);
        
        
        let external_handler = ExternalService::new(&params.auth,&params.kiosk_uid.to_owned(),hub.spoke().clone());
        let external_service = StatefulService::create(external_handler);        
        
        //Don't loop this. If the internal service goes down, need to rebuild entirely.
        let internal_service_future = {
            let spokeclone=spoke.clone();
            let paramsclone=params.clone();
            tokio::spawn(
                async move {
                    let internal_handler = InternalService::new(
                        &paramsclone,
                        std::sync::Arc::new(tokio::sync::Mutex::new(external_command_receiver)), //partial move of hub
                        spokeclone
                        //internal_service_notification_sender.clone()
                    );
                    let internal_service= StatefulService::create(internal_handler);  
                    match internal_service.start(
                        IpAddr::V4(Ipv4Addr::LOCALHOST),
                        (&paramsclone).internal_port.to_owned(),
                        ConnectionProperties{
                            with_upgrades:true,
                            tls:None
                        }
                    ).await
                    {
                        Ok(()) => println!("Internal service exited gracefully. Restarting immediately."),
                        Err(e) => eprintln!("Internal service error: {:?}. Restarting immediately.",e)
                    }
                }
            )
        };

        let external_service_future = match generate_simple_certificates(["*".to_string()])
        {
            Ok(keypair)=>{
                let paramsclone=params.clone();

                /*
                external_service.start(
                    IpAddr::V4(Ipv4Addr::UNSPECIFIED),
                    paramsclone.external_port,
                    ConnectionProperties{
                        with_upgrades:false,
                        tls:Some(keypair)
                    }                    
                )
                */
                
                //Don't loop this. If the external service goes down, rebuild entirely.
                tokio::spawn(
                    async move {
                        match external_service.start(
                            IpAddr::V4(Ipv4Addr::UNSPECIFIED),
                            paramsclone.external_port,
                            ConnectionProperties{
                                with_upgrades:false,
                                tls:Some(keypair)
                            }                    
                        ).await
                        {
                            Ok(()) => println!("Internal service exited gracefully. Restarting immediately."),
                            Err(e) => eprintln!("Internal service error: {:?}. Restarting immediately.",e)
                        }
                    }
                )
                
            },
            Err(e)=>{
                panic!("Couldn't create certificates. {:?}",e);
            }
        };

        let mqtt_client_future = {
            let spokeclone=spoke.clone();
            let paramsclone=params.clone();
            tokio::spawn(
                async move {
                    loop {
                        println!("MQTT client starting.");
                        let client = mqtt::get_has_client(&spokeclone, &paramsclone.mqtt_config, paramsclone.kiosk_uid).await;
                        match client.run().await
                        {
                            Ok(_) => {
                                println!("MQTT client ended gracefully. Restarting immediately.")
                            },
                            Err(e) => {
                                eprintln!("MQTT client error: {:?}. Restarting immediately.",e)
                            },
                        }
                    }
                }
            )
            //tokio::spawn(async move{
            //        let mqtt_client=mqtt::get_has_client(spokeclone, paramsclone.mqtt_config, paramsclone.kiosk_uid).await;
            //        mqtt_client.run().await
            //    }
            //)
        };

        let voice_command_task_join_handle=
        {
            let spokeclone=spoke.clone();
            let paramsclone=params.clone();
            spawn_voice_command_listener(paramsclone,spokeclone)
        };

        let hub_future=tokio::spawn(
            async move{
                loop{
                    match hub.start().await
                    {
                        Ok(()) => println!("Internal service exited gracefully. Restarting immediately."),
                        Err(e) => eprintln!("Internal service error: {:?}. Restarting immediately.",e)
                    }
                }
            }
        );

        println!("Services created.");

        /*
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
        */

        let result:Result<(), tokio::task::JoinError>=tokio::select! {
            result=hub_future=>{eprintln!("Hub service ended."); result},
            result=internal_service_future=>{eprintln!("Internal service ended."); result},
            result=external_service_future=>{eprintln!("External service ended."); result},
            result=mqtt_client_future=>{eprintln!("MQTT service ended."); result},
            result=voice_command_task_join_handle=>{eprintln!("Voice service ended."); result},
        };

        match result
        {
            Ok(_)=>{println!("Faux-Show main loop joined gracefully.")}
            Err(e)=>{eprintln!("Faux-Show main thread joined in error: {:?}",e)}
        };
    }
}
