use std::{error::Error, process::ExitStatus, time::Duration};

use tokio::sync::watch;

pub struct Watchdog
{
    sender:watch::Sender<()>
}

#[derive(Clone)]
pub struct WatchdogNotifier
{
    sender:watch::Sender<()>,
}

impl WatchdogNotifier
{
    pub fn notify_okay(&self)
    {
        self.sender.send_modify(|_|{});
    }
}

pub const WATCHDOG_TIMEOUT:u64=60;
pub const HEARTBEAT_INTERVAL:u64=WATCHDOG_TIMEOUT-10;

impl Watchdog
{
    pub fn new()->Watchdog
    {
        //Watchdog watch
        let (watchdog_sender, watchdog_receiver) = watch::channel::<()>(());

        //Spawn the watch loop once and only once here. It should be infallible.
        tokio::spawn(async move {
            Self::watchloop(watchdog_receiver).await
        });

        Watchdog{
            sender:watchdog_sender
        }
    }

    pub fn get_notifier(&self)->WatchdogNotifier
    {
        WatchdogNotifier{
            sender:self.sender.clone()
        }
    }

    async fn watchloop(mut receiver:watch::Receiver<()>)
    {
        loop {
            let receiver_future=receiver.changed();
            
            match tokio::time::timeout(Duration::from_secs(WATCHDOG_TIMEOUT), receiver_future).await
            {
                Ok(_) => (),
                Err(e) => {
                    //Watchdog timeout failed
                    eprintln!("watchdog.rs: timeout-{:?}",e);
                    match restart_cage_tty1()
                    {
                        Ok(_)=>(),
                        Err(e)=>{
                            eprintln!("watchdog.rs: couldn't restart cage tty");
                            eprintln!("watchdog.rs: {:?}", e);
                        }
                    }
                },
            }
        }
    }
}

fn restart_cage_tty1()->Result<(),Box<dyn Error>>
{
    use std::process::Command;

    //println!("{:?}",std::env::var("PATH"));

    //Have to add path to wlr-randr as packaged rust application path is limited
    let path = "/run/current-system/sw/bin/";

    let mut shell_command = Command::new("sh");
    let command= shell_command
        .env("PATH",path)
        .arg("-c")
        .arg("systemctl restart cage-tty1");

    //get output
    let query_output = command.output();
    
    match query_output
    {
        Ok(query_result)=>{
            if query_result.status.success()
            {
                match std::str::from_utf8(&query_result.stdout)
                {
                    Ok(res)=>println!("watchdog.rs: Command result: {}",res),
                    Err(e)=>{return Err(Box::new(e));}
                };
            }
            else {
                match std::str::from_utf8(&query_result.stderr)
                {
                    Ok(res)=>println!("watchdog.rs: Command error: {}",res),
                    Err(e)=>{return Err(Box::new(e));}
                };                     
            }
        }
        Err(e)=>{
            return Err(Box::new(e));
        }
    };

    Ok(())
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_watchdog_command() {
        restart_cage_tty1().expect("Should return ok");
    }
}