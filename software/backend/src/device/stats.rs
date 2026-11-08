
//Might use command:
//nmcli -f IN-USE,SIGNAL device wifi | grep '*'
//To get wifi signal quality and put it on the display.
//Returns
//*            57
//Can get number back and put it on screen

//CPU utilization too?

use std::error::Error;

pub(crate) fn get_wifi_strength()->Option<u32>
{
    use std::process::Command;

    //Have to add path to wlr-randr as packaged rust application path is limited
    let path = "/run/current-system/sw/bin/";

    let query_output = Command::new("sh")
        .env("PATH",path)
        .arg("-c")
        .arg("nmcli -f IN-USE,SIGNAL device wifi | grep '*'")
        .output();
    
    match query_output
    {
        Ok(query_result)=>{
            if query_result.status.success()
            {
                match std::str::from_utf8(&query_result.stdout)
                {
                    Ok(res)=>{
                        println!("stats.rs: Command result: {}",res);
                        let number_string=res.trim_matches(['*',' ']);
                        match number_string.parse::<u32>()
                        {
                            Ok(number) => {
                                if number <= 100
                                {
                                    Some(number)
                                }
                                else {
                                    eprintln!("stats.rs: Wifi strength {} is out of range.",number);
                                    None
                                }
                            },
                            Err(e) => {
                                eprintln!("stats.rs: Parse error: {:?}",e);
                                None
                            }
                        }
                    },
                    Err(e)=>{
                        eprintln!("stats.rs: String error: {:?}",e);
                        None
                    }
                }
            }
            else {
                match std::str::from_utf8(&query_result.stderr)
                {
                    Ok(res)=>{
                        println!("stats.rs: Command error: {}",res)
                    },
                    Err(e)=>{
                        eprintln!("stats.rs: String error: {:?}",e);
                    }
                };                     
                None
            }
        }
        Err(e)=>{
            eprintln!("Couldn't query video outputs.");
            None
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_states() {
        get_wifi_strength().expect("Should return ok");
    }
}