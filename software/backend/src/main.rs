use hyper_services::request_processing::Auth;
use faux_show_backend::InitializationParameters;


const DEV_MODE_ENV_KEY:&str="DEVELOPMENT_MODE";

const EXTERNAL_USER_ENV_KEY:&str="EXTERNAL_USER";
const EXTERNAL_PASSWORD_ENV_KEY:&str="EXTERNAL_PASSWORD";

const KIOSK_USER_ID_ENV_KEY:&str="KIOSK_USER_ID";

const DEVICE_MQTT_NAME_ENV_KEY:&str="DEVICE_NAME";
const DEVICE_MQTT_ID_ENV_KEY:&str="DEVICE_ID";

const PHOTOPRISM_KEY_ENV_KEY:&str="PHOTOPRISM_KEY";

const WHISPER_SERVER_URL_KEY:&str="WHISPER_SERVER";
const WAKEWORD_ONNX_FILE_KEY:&str="WAKEWORD_ONNX_FILE";

const PROD_INTERNAL_SERVICE_DIR:&str="/var/www/internal";
const PROD_CONFIG_DIR:&str="/var/www/config";
const PROD_INTERNAL_PORT:u16=30125;
const PROD_EXTERNAL_PORT:u16=443;

const DEV_INTERNAL_SERVICE_DIR:&str="../frontend/build";
const DEV_CONFIG_DIR:&str="../dev";
const DEV_INTERNAL_PORT:u16=PROD_INTERNAL_PORT;
const DEV_EXTERNAL_PORT:u16=8443;

#[tokio::main]
async fn main() {
  
    let dev_mode:bool = match std::env::var(DEV_MODE_ENV_KEY)
    {
        Ok(val)=>{
            val.to_lowercase()=="true"
        },
        Err(_)=>{
            false
        }
    };

    let device_mqtt_name:String = match std::env::var(DEVICE_MQTT_NAME_ENV_KEY)
    {
        Ok(val)=>{
            val
        },
        Err(_)=>{
            eprintln!("Must provide mqtt device name as an environment variable.");
            return;
        }
    };

    let device_mqtt_id:String = match std::env::var(DEVICE_MQTT_ID_ENV_KEY)
    {
        Ok(val)=>{
            val
        },
        Err(_)=>{
            eprintln!("Must provide mqtt device id as an environment variable.");
            return;
        }
    };

    let user:String = match std::env::var(EXTERNAL_USER_ENV_KEY)
    {
        Ok(val)=>{
            val
        },
        Err(_)=>{
            eprintln!("Must provide external username as an environment variable.");
            return;
        }
    };

    let password:String = match std::env::var(EXTERNAL_PASSWORD_ENV_KEY)
    {
        Ok(val)=>{
            val
        },
        Err(_)=>{
            eprintln!("Must provide external password as an environment variable.");
            return;
        }
    };

    let kiosk_uid:u64 = match std::env::var(KIOSK_USER_ID_ENV_KEY)
    {
        Ok(val)=>{
            match val.parse()
            {
                Ok(val)=>val,
                Err(e)=>{
                    eprintln!("{:?}",e);
                    eprintln!("kiosk uid is not a valid unsigned integer.");
                    return;
                }
            }
        },
        Err(_)=>{
            eprintln!("Must provide kiosk user id as an environment variable.");
            return;
        }
    };

    let photoprism_key:String = match std::env::var(PHOTOPRISM_KEY_ENV_KEY)
    {
        Ok(val)=>{
            val
        },
        Err(_)=>{
            eprintln!("Must provide photoprism key as an environment variable.");
            return;
        }
    };

    let auth:Auth=Auth{
        user,
        password
    };

    let whisper_server_url:String = match std::env::var(WHISPER_SERVER_URL_KEY)
    {
        Ok(val)=>{
            val
        },
        Err(_)=>{
            eprintln!("Must provide whisper server url as an environment variable.");
            return;
        }
    };

    let wakeword_onnx_file:String = match std::env::var(WAKEWORD_ONNX_FILE_KEY)
    {
        Ok(val)=>{
            val
        },
        Err(_)=>{
            eprintln!("Must provide wakeword onnx file path as an environment variable.");
            return;
        }
    };
    

    let mqtt_config = faux_show_backend::mqtt::MQTTConfiguration
    {
        id: device_mqtt_id.clone(),
        name: device_mqtt_name,
        origin_name: "Tyler Moore".to_string(),
        origin_sw: "0.1.0".to_string(),
        client_id: device_mqtt_id.to_string()+"_client",
        server_url: "10.10.10.10".to_string(),
        server_port: 1883,
        object_id: device_mqtt_id.to_string(),
        discovery_prefix: has_mqtt::mqtt_client::DEFAULT_DISCOVERY_PREFIX.to_string(),
    };

    let params:InitializationParameters=match dev_mode
    {
        true=>{
            println!("Running in development mode.");
            InitializationParameters::new(
                DEV_INTERNAL_SERVICE_DIR,
                DEV_CONFIG_DIR,
                DEV_INTERNAL_PORT,
                DEV_EXTERNAL_PORT,
                auth,
                kiosk_uid,
                mqtt_config,
                photoprism_key,
                whisper_server_url,
                wakeword_onnx_file
            )
        },
        false=>{
            println!("Running in production mode.");
            InitializationParameters::new(
                PROD_INTERNAL_SERVICE_DIR,
                PROD_CONFIG_DIR,
                PROD_INTERNAL_PORT,
                PROD_EXTERNAL_PORT,
                auth,
                kiosk_uid,
                mqtt_config,
                photoprism_key,
                whisper_server_url,
                wakeword_onnx_file
            )
        }
    };

    println!("Init params {:?}",&params);
    faux_show_backend::start_and_run(params).await
}