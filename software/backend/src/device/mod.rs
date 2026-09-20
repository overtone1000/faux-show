use std::error::Error;

const POSSIBLE_DISPLAYS:[&str;2]=[
    "HDMI-A-1",
    "HDMI-A-2"
];

pub(crate) fn set_screen_state(screen_on:bool, kiosk_uid:&u64)->Result<(),Box<dyn Error>>
{
    use std::process::Command;

    let arg = match screen_on
    {
        true=>"--on",
        false=>"--off"
    };

    //println!("{:?}",std::env::var("PATH"));

    //Have to add path to wlr-randr as packaged rust application path is limited
    let path = "/run/current-system/sw/bin/";

    //run directory for kiosk user, which is the user for the cage session
    let xdg_runtime_dir = "/run/user/".to_owned()+&kiosk_uid.to_string();

    //get output
    let query_command = "wlr-randr".to_owned();
    let query_output = Command::new("sh")
        .env("XDG_RUNTIME_DIR", &xdg_runtime_dir)
        .env("WAYLAND_DISPLAY", "wayland-0")
        .env("PATH",path)
        .arg("-c")
        .arg(query_command)
        .output();

    let mut all_results:Vec<Result<_,std::io::Error>>=Vec::new();
    
    match query_output
    {
        Ok(query_result)=>{
            let result_as_str = match std::str::from_utf8(&query_result.stdout)
            {
                Ok(res)=>res,
                Err(e)=>{return Err(Box::new(e));}
            };  
            for display in POSSIBLE_DISPLAYS
            {
                if result_as_str.contains(display)
                {
                    //command to control display
                    let command = "wlr-randr --output ".to_owned() + display + " " + arg;

                    all_results.push(
                        Command::new("sh")
                            .env("XDG_RUNTIME_DIR", &xdg_runtime_dir)
                            .env("WAYLAND_DISPLAY", "wayland-0")
                            .env("PATH",path)
                            .arg("-c")
                            .arg(command)
                            .output()
                    );
                }
            }
        }
        Err(e)=>{
            eprintln!("Couldn't query video outputs.");
            return Err(Box::new(e));
        }
    };

    for result in all_results
    {
        match result
        {
            Ok(_)=>(),
            Err(e)=>{
                return Err(Box::new(e));
            }
        }
    };

    Ok(())
}