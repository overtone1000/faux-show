use std::thread::current;

use serde::{Deserialize, Serialize};

use crate::{comm::{CommunicationSpoke, external_commands::{AutoTabConfig, Command}}, voice::{text_manip::{MatchResult, check_for_match_string, extract_timer_init}, whisper_streaming::{LivekitTranscriptionSegment, LivekitTranscriptionWord}}};


#[derive(Deserialize,Serialize, PartialEq, Eq, Debug, Clone)]
pub enum VoiceCommandMode
{
    Contains(String),
    Timer
}

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
                best_command.run(spoke, best_match);
                true //Matched, return true
            },
            None=>false //No match, return false
        }
    }
}

impl VoiceCommand
{
    
    pub fn run(&self, spoke:&CommunicationSpoke, match_result:MatchResult){
        match &self.mode
        {
            VoiceCommandMode::Contains(_) => {
                match &self.action{
                    Some(action) => action.run(spoke),
                    None => eprintln!("Not yet implemented."),
                }
            },
            VoiceCommandMode::Timer => {
                //doesn't use action
                //need to parse out name and time from timer
                eprintln!("Need to do more here!");
                extract_timer_init(match_result);
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
                check_for_match_string("timer".to_string(),segments)
            },
        }
    }
}

impl VoiceCommandAction
{
    fn run(&self, spoke:&CommunicationSpoke){
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
                );
            },
            VoiceCommandAction::AcknowledgeAlarms(ack) => {
                spoke.send_external_command(
                    Command::AcknowledgeAlarms(*ack)
                );
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
}