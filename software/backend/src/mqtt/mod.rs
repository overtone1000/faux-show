use std::collections::HashMap;

use has_mqtt::{component::HomeAssistantDeviceComponent, device::HomeAssistantDeviceConfiguration, mqtt_client::{DEFAULT_DISCOVERY_PREFIX, HASMQTTClient}, platform::{switch::{component::Switch, state::SwitchState}, text::component::Text}};
use hyper_util::server::conn::auto;
use tokio::sync::{broadcast, mpsc::UnboundedSender};

use crate::comm::{CommunicationHub, CommunicationSpoke, external_commands::{AutoTabConfig, Command}, internal_notifications::InternalServiceNotification};

#[derive(Debug)]
pub struct MQTTConfiguration
{
    pub id:String,
    pub name:String,
    pub origin_name:String,
    pub origin_sw:String,
    pub client_id:String,
    pub server_url:String,
    pub server_port:u16,
    pub object_id:String,
    pub discovery_prefix:String
}

pub async fn get_has_client(spoke:CommunicationSpoke, config:&MQTTConfiguration, kiosk_uid:u64)->HASMQTTClient
{

    let mut cmps_hm:HashMap<String,HomeAssistantDeviceComponent> = HashMap::new();

    let mut add_cmp = |kvp:(String,HomeAssistantDeviceComponent)|->(){
        cmps_hm.insert(kvp.0,kvp.1);
    };

    add_cmp(monitor_switch(&config.id, &config.name, kiosk_uid, spoke.clone()));
    add_cmp(auto_tab_set(&config.id, &config.name, spoke));

    let device=HomeAssistantDeviceConfiguration::new(
        config.id.to_string(),
        config.name.to_string(),
        config.origin_name.to_string(),
        config.origin_sw.to_string(),
        cmps_hm
    );


    HASMQTTClient::start(
        &config.client_id,
        &config.server_url,
        config.server_port,
        &config.discovery_prefix,
        &config.object_id,
        device
    ).await
}

fn monitor_switch(
    device_id:&str, 
    device_name:&str, 
    kiosk_uid:u64, 
    spoke:CommunicationSpoke
)->(String,HomeAssistantDeviceComponent)
{
    let handle_state_change =move |state:SwitchState|->Option<SwitchState>
    {
        let command:Command=Command::SetScreenState(state.as_bool());
        spoke.send_external_command(command);
        
        let notification:InternalServiceNotification=InternalServiceNotification::Sleep(state.as_bool());
        spoke.notify_of_state_change(notification);

        match crate::device::set_screen_state(state.as_bool(),&kiosk_uid)
        {
            Ok(_)=>Some(state),
            Err(e)=>{
                eprintln!("Error setting screen state. {:?}",e);  
                None
            }
        }
    };

    (
        "monitor".to_string(),
        Switch::new(
            device_id,
            device_name,
            "monitor",
            "Monitor",
            Box::new(handle_state_change)
        )
    )
}

fn auto_tab_set(
    device_id:&str,
    device_name:&str,
    spoke:CommunicationSpoke
)->(String,HomeAssistantDeviceComponent)
{
    let handle_state_change =move |tab_config:String|->Option<String>
    {
        match serde_json::from_str::<AutoTabConfig>(&tab_config)
        {
            Ok(autotabdata) => {
                if spoke.send_external_command(Command::AutoTab(autotabdata))
                {
                    Some(tab_config)
                }
                else {
                    None            
                }
            },
            Err(e) => {eprintln!("{:?}",e); None},
        }       
    };

    (
        "auto_tab".to_string(),
        Text::new(
            device_id,
            device_name,
            "auto_tab",
            "Auto Tab",
            Box::new(handle_state_change)
        )
    )
}