use serde::{Deserialize, Serialize, de::Visitor};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct ChangeDashData {url:String}

#[derive(Debug, PartialEq, Clone)]
pub enum VoiceControlState
{
    NotEnabled=0,
    ListeningForWakeword=1,
    StreamingToWhisper=2
}
impl Serialize for VoiceControlState
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer {

        let i:i32 = match self
        {
            VoiceControlState::NotEnabled => 0,
            VoiceControlState::ListeningForWakeword => 1,
            VoiceControlState::StreamingToWhisper => 2,
        };

        serializer.serialize_i32(i)
    }
}

struct VoiceControlStateVisitor;

impl<'de> Visitor<'de> for VoiceControlStateVisitor {
    type Value = VoiceControlState;
    
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("an integer")
    }

    fn visit_i32<E>(self, value:i32)->Result<Self::Value,E>
    where
        E: serde::de::Error,
    {
        match value {
            0=>Ok(VoiceControlState::NotEnabled),
            1=>Ok(VoiceControlState::ListeningForWakeword),
            2=>Ok(VoiceControlState::StreamingToWhisper),
            i=>{
                //Err(std::io::Error::new(std::io::ErrorKind::InvalidData,format!("{i}",i)))
                Err(serde::de::Error::custom(format!("Unexpected voice control state value {}",i)))
            }
        }
    }
    
}

impl<'de> Deserialize<'de> for VoiceControlState
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de> {
            
        deserializer.deserialize_i32(VoiceControlStateVisitor)
    }
}


#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum Command
{
    AutoTab(String),
    PhotoprismKey(String),
    SetScreenState(bool),
    SetVoiceControlState(VoiceControlState)
}

#[cfg(test)]
mod tests {

    use super::*;

    fn check_serialization(command: &Command) {
        println!("Serialization test:");
        let serialized = serde_json::to_string(command).expect("Should serialize.");
        println!("   {}", serialized);
        let deserialized: Command = serde_json::from_str(&serialized).expect("Should deserialize.");
        println!("   {:?}", deserialized);
        assert_eq!(*command,deserialized)        
    }

    #[test]
    fn serialization() {
        check_serialization(&Command::AutoTab( "https://www.example.com".to_string()));
        check_serialization(&Command::SetScreenState(true));
    }
}