use std::thread::current;

use serde::{Deserialize, Serialize};

use crate::{comm::{CommunicationSpoke, external_commands::{AutoTabConfig, Command}}, voice::{text_manip::{HOUR, HOURS, MINUTE, MINUTES, SECOND, SECONDS, MatchResult, check_for_match_string, extract_timer_init}, whisper_streaming::{LivekitTranscriptionSegment, LivekitTranscriptionWord}}};


#[derive(Deserialize,Serialize, PartialEq, Eq, Debug, Clone)]
pub enum VoiceCommandMode
{
    Contains(String),
    Timer,
    Slideshow
}

pub(crate) const TIMER_START:&str="timer";
pub(crate) const TIMER_END:&str="engage";
pub(crate) const SLIDESHOW:&str="slideshow";

#[derive(Deserialize,Serialize, PartialEq, Eq, Debug, Clone)]
pub enum VoiceCommandAction
{
    OpenPage(String),
    AcknowledgeAlarms(bool)
}

#[derive(Deserialize,Serialize, PartialEq, Eq, Debug, Clone)]
pub struct VoiceCommand
{
    mode:VoiceCommandMode,
    action:Option<VoiceCommandAction>
}

#[derive(Deserialize,Serialize, PartialEq, Eq, Debug, Clone)]
pub struct VoiceCommandList{
    commands:Vec<VoiceCommand>
}

impl VoiceCommandList {

    pub fn get_hotwords(&self)->String
    {
        let mut retval="".to_string();
        for vc in self.commands.iter()
        {
            match &vc.mode
            {
                VoiceCommandMode::Contains(matchwords) => {
                    for word in matchwords.split(" ")
                    {
                        retval+=word;
                        retval+=",";
                    }
                },
                VoiceCommandMode::Timer => {
                    retval+=TIMER_START;
                    retval+=",";
                    retval+=TIMER_END;
                    retval+=",";
                    retval+=HOUR;
                    retval+=",";
                    retval+=HOURS;
                    retval+=",";
                    retval+=MINUTE;
                    retval+=",";
                    retval+=MINUTES;
                    retval+=",";
                    retval+=SECOND;
                    retval+=",";
                    retval+=SECONDS;
                    retval+=",";
                },
                VoiceCommandMode::Slideshow => {
                    retval+=SLIDESHOW;
                    retval+=",";
                }
            };
        }
        retval=retval.trim_matches([',',' ']).to_lowercase().to_string();
        println!("voice_command.rs: Hotwords are: {}",retval);
        retval
    }

    pub fn check_for_match_and_run_best_match(
        &self,
        segments:&Vec<LivekitTranscriptionSegment>,
        spoke:&CommunicationSpoke
    )->bool
    {
        let mut best_match:Option<(&VoiceCommand,MatchResult)>=None;

        println!("Checking for match of {} possible commands.", self.commands.len());

        for current_command in &self.commands
        {
            match current_command.mode.check_for_match(segments)
            {
                Some(current_match)=>{
                    match &best_match
                    {
                        Some((_previous_best_command,previous_best_match))=>{
                            if current_match.score>previous_best_match.score
                            {
                                best_match=Some((current_command,current_match));
                            }
                        },
                        None=>{best_match=Some((current_command,current_match));}
                    }
                },
                None=>()
            }
        }

        match best_match
        {
            Some((best_command,best_match))=>
            {
                println!("Best match with score {}: {:?}",best_match.score,best_command);
                //Matched, return result of run
                best_command.run(spoke, best_match)
            },
            None=>false //No match, return false
        }
    }
}

impl VoiceCommand
{
    
    pub fn run(&self, spoke:&CommunicationSpoke, match_result:MatchResult)->bool{
        match &self.mode
        {
            VoiceCommandMode::Contains(_) => {
                match &self.action{
                    Some(action) => action.run(spoke),
                    None => false,
                }
            },
            VoiceCommandMode::Timer => {
                match extract_timer_init(match_result)
                {
                    Some(timer)=>{
                        spoke.send_external_command(Command::NewTimer(timer))
                    },
                    None=>false
                }
            },
            VoiceCommandMode::Slideshow => {
                spoke.send_external_command(Command::ShowSlideshow(true))
            },
        }
    }
}

impl VoiceCommandMode
{

    fn check_for_match(&self, segments:&Vec<LivekitTranscriptionSegment>)->Option<MatchResult>
    {
        println!("Checking for match in {:?}",segments);
        match self
        {
            VoiceCommandMode::Contains(match_string) => {
                check_for_match_string(match_string.to_string(),segments)
            },
            VoiceCommandMode::Timer => {
                println!("voice_command.rs: Checking for timer match.");
                //Command is "timer $NAME blah hours blah minutes blah seconds engage"
                if check_for_match_string(TIMER_END.to_string(),segments).is_some()
                {
                    check_for_match_string(TIMER_START.to_string() ,segments)
                }
                else {
                    None
                }
            },
            VoiceCommandMode::Slideshow => {
                check_for_match_string(SLIDESHOW.to_string(),segments)
            },
        }
    }
}

impl VoiceCommandAction
{
    fn run(&self, spoke:&CommunicationSpoke)->bool{
        match self
        {
            VoiceCommandAction::OpenPage(url) => {
                spoke.send_external_command(
                    Command::AutoTab(
                        AutoTabConfig{
                            url:url.to_string(),
                            priority:3,
                            timeout_seconds:5*60
                        }                        
                    )
                )
            },
            VoiceCommandAction::AcknowledgeAlarms(ack) => {
                spoke.send_external_command(
                    Command::AcknowledgeAlarms(*ack)
                )
            },
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialization_and_deserialization() {
        let test_commands = 
            VoiceCommandList {
                commands:[
                    VoiceCommand {
                        mode:VoiceCommandMode::Contains("example page".to_string()),
                        action:Some(VoiceCommandAction::OpenPage("http://www.example.com".to_string()))
                    }
                ].to_vec()
            }
        ;
        
        let serialized = serde_json::to_string_pretty(&test_commands).expect("Should serialize.");
        println!("{}",serialized);
        let deserialized = serde_json::from_str::<VoiceCommandList>(&serialized).expect("Should deserialize.");
        assert_eq!(test_commands,deserialized);
    }

    #[test]
    fn voice_timer(){
        let vct = VoiceCommand{
            mode:VoiceCommandMode::Timer,
            action:None
        };

        let vcl=VoiceCommandList{
            commands:vec![vct]
        };

        let (hub, _receiver)=crate::comm::CommunicationHub::new();

        let segments = vec![
            LivekitTranscriptionSegment::create_synthetic("timer one  minute,  15  seconds.  Engage.", 1000f32)
        ];

        vcl.check_for_match_and_run_best_match(
            &segments,
            hub.spoke()
        );
    }
}