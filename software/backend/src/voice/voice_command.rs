use serde::{Deserialize, Serialize};

const CALENDAR:&str = "calendar";
const PHOTOS:&str = "photos";
const TASKS:&str = "tasks";
const CAMERAS:&str = "cameras";
const TELL_HOME_ASSISTANT:&str = "tell home assistant";

#[derive(PartialEq,Debug)]
pub enum VoiceCommand
{
    Calendar,
    Photos,
    Tasks,
    Cameras,
    TellHomeAssistant(String)
}

#[derive(Serialize,Deserialize,Debug)]
pub struct RawVoiceCommand
{
    command:String,
    args:Option<String>
}


impl <'de> Deserialize<'de> for VoiceCommand
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de> {
            match RawVoiceCommand::deserialize(deserializer)
            {
                Ok(raw_command) => {
                    match raw_command.command.as_str()
                    {
                        CALENDAR => Ok(VoiceCommand::Calendar),
                        PHOTOS => Ok(VoiceCommand::Photos),
                        TASKS => Ok(VoiceCommand::Tasks),
                        CAMERAS => Ok(VoiceCommand::Cameras),
                        TELL_HOME_ASSISTANT => {
                            match raw_command.args
                            {
                                Some(args)=>{
                                    Ok(VoiceCommand::TellHomeAssistant(args))
                                },
                                None=>Err(format!("'{:?}' is not a valid command", raw_command)).map_err(serde::de::Error::custom)
                            }
                        },
                        _ => Err(format!("'{:?}' is not a valid command", raw_command)).map_err(serde::de::Error::custom)
                    }
                },
                Err(e) => Err(e),
            }
    }
}

impl Serialize for VoiceCommand
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer {

        let str= match self
        {
            VoiceCommand::Calendar => CALENDAR,
            VoiceCommand::Photos => PHOTOS,
            VoiceCommand::Tasks => TASKS,
            VoiceCommand::Cameras => CAMERAS,
            VoiceCommand::TellHomeAssistant(_)=> TELL_HOME_ASSISTANT,
        };

        let args = match self
        {
            VoiceCommand::TellHomeAssistant(message)=> Some(message.to_string()),
            _ => None
        };

        let raw_command= RawVoiceCommand{
            command:str.to_string(),
            args:args
        };

        raw_command.serialize(serializer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialization_and_deserialization() {
        let test_commands = [
            VoiceCommand::Calendar,
            VoiceCommand::Photos,
            VoiceCommand::Tasks,
            VoiceCommand::Cameras,
            VoiceCommand::TellHomeAssistant("clean the whole house".to_string()),
        ];
        
        for test_command in test_commands
        {
            let serialized = serde_json::to_string_pretty(&test_command).expect("Should serialize.");
            println!("{}",serialized);
            let deserialized = serde_json::from_str::<VoiceCommand>(&serialized).expect("Should deserialize.");
            assert_eq!(test_command,deserialized);
        }
    }
}