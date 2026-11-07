use std::{error::Error, process::ExitStatus};


pub(crate) fn restart_cage_tty1()->Result<(),Box<dyn Error>>
{
    use std::process::Command;

    //println!("{:?}",std::env::var("PATH"));

    //Have to add path to wlr-randr as packaged rust application path is limited
    let path = "/run/current-system/sw/bin/";

    let mut shell_command = Command::new("sh");
    let command= shell_command
        .env("PATH",path)
        .arg("-c")
        .arg("systemctl status cage-tty1");

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