pub(crate) mod services;


use std::
    net::{IpAddr, Ipv4Addr}
;

use hyper_services::service::stateless_service::StatelessService;
use hyper_services::spawn_server;

use crate::services::internal::InternalService;

const INTERNAL_SERVICE_DIR:&str="/var/www/internal";
const INTERNAL_PORT:u16=30125;
const EXTERNAL_PORT:u16=443;

pub async fn start_and_run() {
    loop {
        
        println!("Starting services.");

        //Create event servers
        let internal_service = {

            let internal_service:StatelessService<InternalService>=StatelessService::create();

            spawn_server(
                IpAddr::V4(Ipv4Addr::LOCALHOST),
                INTERNAL_PORT,
                internal_service,
            )
        };

        let external_service = {

            let external_service:StatelessService<InternalService>=StatelessService::create();

            spawn_server(
                IpAddr::V4(Ipv4Addr::UNSPECIFIED),
                EXTERNAL_PORT,
                external_service,
            )
        };

        println!("Services running.");

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
