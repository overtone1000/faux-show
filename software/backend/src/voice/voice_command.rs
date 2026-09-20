use std::thread::current;

use serde::{Deserialize, Serialize};

use crate::{comm::{CommunicationSpoke, external_commands::Command}, voice::whisper_streaming::LivekitTranscriptionSegment};


#[derive(Deserialize,Serialize, PartialEq, Eq, Debug, Clone)]
pub enum VoiceCommandMode
{
    Contains(String),
    Timer
}

#[derive(Deserialize,Serialize, PartialEq, Eq, Debug, Clone)]
pub enum VoiceCommandAction
{
    OpenPage(String)
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
        let mut best_match:Option<(&VoiceCommand,f32)>=None;

        println!("Checking for match of {} possible commands.", self.commands.len());

        for current_command in &self.commands
        {
            match current_command.mode.check_for_match(segments)
            {
                Some(current_score)=>{
                    match &best_match
                    {
                        Some((_previous_best_command,previous_best_score))=>{
                            if current_score>*previous_best_score
                            {
                                best_match=Some((current_command,current_score));
                            }
                        },
                        None=>{best_match=Some((current_command,current_score));}
                    }
                },
                None=>()
            }
        }

        match best_match
        {
            Some((best_command,score))=>
            {
                println!("Best match with score {}: {:?}",score,best_command);
                best_command.run(spoke);
                true //Matched, return true
            },
            None=>false //No match, return false
        }
    }
}

impl VoiceCommand
{
    pub fn run(&self, spoke:&CommunicationSpoke){
        match &self.mode
        {
            VoiceCommandMode::Contains(_) => {
                match &self.action{
                    Some(action) => action.run(spoke),
                    None => eprintln!("Not yet implemented."),
                }
            },
            VoiceCommandMode::Timer => eprintln!("Not yet implemented."),
        }
    }
}

impl VoiceCommandMode
{
    fn check_for_match(&self, segments:&Vec<LivekitTranscriptionSegment>)->Option<f32>
    {
        match self
        {
            VoiceCommandMode::Contains(match_string) => {
                let words:Vec<&str>=match_string.split(" ").collect();
                
                let tidied_words:Vec<String>=words.iter().map(
                    |word|{
                        word.trim().to_lowercase()
                    }
                ).collect();

                println!("Tidied words: {:?}",tidied_words);

                let mut scores:Vec<f32>=Vec::new();
                
                let mut current_word_index=0;

                for segment in segments
                {
                    match &segment.words
                    {
                        Some(tswords) => {
                            for tsword in tswords
                            {
                                match tidied_words.get(current_word_index)
                                {
                                    Some(current_word) => {
                                            println!("   current_word: \"{}\"", current_word);
                                            let tsword_cleaned = tsword.word.to_lowercase().trim().trim_matches(['?','!','.']).to_string();
                                            println!("   tsword_cleaned: \"{}\"", tsword_cleaned);
                                            if tsword_cleaned == *current_word
                                            {
                                                println!("Match!");
                                                scores.push(tsword.probability);
                                                current_word_index+=1;
                                                if current_word_index==tidied_words.len()
                                                {
                                                    //All words found, calculate geometric mean and return!
                                                    println!("Finished, returning.");
                                                    return Some(f32::powf(scores.iter().product(),1.0/(scores.len() as f32)));
                                                }
                                            }
                                        }
                                    None => eprintln!("Shouldn't be reachable. Possible an index error in nearby block."),
                                };
                            }
                        },
                        None => (),
                    }
                }
            },
            VoiceCommandMode::Timer => eprintln!("Not yet implemented."),
        };

        None
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
                        url.to_string()
                    )
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