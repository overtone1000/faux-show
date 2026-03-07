use shmashmexa_backend::InitializationParameters;


const DEV_MODE_ENV_KEY:&str="DEVELOPMENT_MODE";

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

    //Currently doesn't do anything. May remove later.
    let dev_mode:bool = match std::env::var(DEV_MODE_ENV_KEY)
    {
        Ok(val)=>{
            val.to_lowercase()=="true"
        },
        Err(_)=>{
            false
        }
    };

    let params:InitializationParameters=match dev_mode
    {
        true=>{
            println!("Running in development mode.");
            InitializationParameters::new(DEV_INTERNAL_SERVICE_DIR,DEV_CONFIG_DIR,DEV_INTERNAL_PORT,DEV_EXTERNAL_PORT)
        },
        false=>{
            println!("Running in production mode.");
            InitializationParameters::new(PROD_INTERNAL_SERVICE_DIR,PROD_CONFIG_DIR,PROD_INTERNAL_PORT,PROD_EXTERNAL_PORT)
        }
    };

    println!("Init params {:?}",&params);
    shmashmexa_backend::start_and_run(params).await
}