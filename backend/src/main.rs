
const DEV_MODE_ENV_KEY:&str="DEVELOPMENT_MODE";

#[tokio::main]
async fn main() {

    //Currently doesn't do anything. May remove later.
    let development_mode = match std::env::var(DEV_MODE_ENV_KEY)
    {
        Ok(val)=>{
            println!("Running in development mode.");
            val.to_lowercase()=="true"
        },
        Err(_)=>{
            println!("Running in production mode.");
            false
        }
    };

    shmashmexa_backend::start_and_run().await
}