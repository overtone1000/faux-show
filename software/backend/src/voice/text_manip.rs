use futures_util::SinkExt;
use serde::{Deserialize, Serialize};
use text2num::find_numbers;

use crate::voice::whisper_streaming::{LivekitTranscriptionSegment, LivekitTranscriptionWord};

pub(crate) const HOUR:&str = "hour";
pub(crate) const HOURS:&str = "hours";
pub(crate) const MINUTE:&str = "minute";
pub(crate) const MINUTES:&str = "minutes";
pub(crate) const SECOND:&str = "second";
pub(crate) const SECONDS:&str = "seconds";

pub fn clean_tsword(tsword:&LivekitTranscriptionWord)->String
{
    tsword.word.to_lowercase().trim().trim_matches(['?','!','.',',',';']).to_string()
}

pub fn clean_match_string_into_words(match_string:String)->Vec<String>
{
    let match_string_words:Vec<&str>=match_string.split(" ").collect();
            
    match_string_words.iter().map(
        |word|{
            word.trim().to_lowercase()
        }
    ).collect()
}

pub struct MatchResult
{
    pub score:f32,
    pub complete:bool,
    pub remaining_tokens:Vec<LivekitTranscriptionWord>
}

pub fn check_for_match_string(match_string:String, segments:&Vec<LivekitTranscriptionSegment>)->Option<MatchResult>
{
    let tidied_match_string_words:Vec<String>=clean_match_string_into_words(match_string);

    //println!("Tidied words: {:?}",tidied_words);

    let mut scores:Vec<f32>=Vec::new();
    
    let mut current_match_word_index=0;

    let complete = {
        let mut complete = true;
        for segment in segments
        {
            if !segment.completed{
                complete=false;
                break;
            }
        }
        complete
    };

    for segment in segments
    {
        match &segment.words
        {
            Some(tswords) => {
                for (index,tsword) in tswords.iter().enumerate()
                {
                    match tidied_match_string_words.get(current_match_word_index)
                    {
                        Some(current_word) => {
                                //println!("   current_word: \"{}\"", current_word);
                                let tsword_cleaned = clean_tsword(tsword);
                                //println!("   tsword_cleaned: \"{}\"", tsword_cleaned);
                                println!("     comparing {} and {}",current_word,tsword_cleaned);
                                if tsword_cleaned == *current_word
                                {
                                    println!("Match!");
                                    scores.push(tsword.probability);
                                    current_match_word_index+=1;
                                    if current_match_word_index==tidied_match_string_words.len()
                                    {
                                        //All words found, return!
                                        println!("Finished, returning.");
                                        
                                        let mut remaining_tokens=Vec::new();
                                        for n in index+1..tswords.len()
                                        {
                                            match tswords.get(n)
                                            {
                                                Some(tsword)=>remaining_tokens.push(tsword.clone()),
                                                None=>()
                                            }
                                        }
                                        
                                        return Some(MatchResult{
                                                score:f32::powf(scores.iter().product(),1.0/(scores.len() as f32)),
                                                remaining_tokens,
                                                complete
                                            }
                                        );
                                    }
                                }
                            }
                        None => eprintln!("Shouldn't be reachable. Possibly an index error in nearby block."),
                    };
                }
            },
            None => (),
        }
    }

    None
}

pub fn concat_words(words:&[String])->String{
    let mut retval = "".to_string();
    for word in words
    {
        retval += " ";
        retval += word.as_str();
    }
    retval.to_string()
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Timer
{
    name:String,
    millis:u32,
    paused:bool
}
pub fn extract_timer_init(match_result:MatchResult)->Option<Timer>
{
    let millis_per_second = 1000.0;
    let millis_per_minute = millis_per_second*60.0;
    let millis_per_hour= millis_per_minute*60.0;

    let mut millis:f64=0.0;

    println!("text_manip.rs: Raw tokens: {:?}",match_result.remaining_tokens);

    let cleaned_tokens={
        let mut ret:Vec<LivekitTranscriptionWord>=Vec::with_capacity(match_result.remaining_tokens.len());
        for mut token in match_result.remaining_tokens
        {
            token.word=clean_tsword(&token);
            ret.push(token);
        }
        ret
    };

    println!("text_manip.rs: Cleaned tokens: {:?}",cleaned_tokens);


    let occurrences = {
        let mut retval:Vec<text2num::Occurence>=Vec::new();
        let tokens= cleaned_tokens.clone();
        print!("text_manip.rs: Cleaned tokens:");
        for n in 0..tokens.len() 
        {
            let token=tokens.get(n).expect("Should exist!");
            match token.word.as_str().parse::<f64>()
            {
                Ok(result)=>{
                    let occ = text2num::Occurence{
                        start: n,
                        end: n+1,
                        text: token.word.clone(),
                        value: result,
                        is_ordinal: false,
                    };
                    retval.push(occ);
                },
                Err(_e)=>()
            };
        }
        println!();

        retval.append(&mut find_numbers(cleaned_tokens.clone().into_iter(), &(text2num::Language::english()), 0.0));

        retval.sort_by(
            |one, two|
            {
                one.start.cmp(&two.start)
            }
        );

        retval
    };

    println!("text_manip.rs: Occurences: {:?}",occurrences);

    let name ={
        if occurrences.len()>0
        {
            //Make the name everything before the first number
            let mut retval="".to_string();

            for n in 0..occurrences.get(0).expect("Should exist.").start
            {
                retval+=&cleaned_tokens.get(n).expect("Should exist.").word;
            }

            retval
        }
        else {
            "Unnamed Timer".to_string()
        }
    };

    for n in 0..occurrences.len()
    {
        let occurence=match occurrences.get(n){
            Some(o)=>o,
            None=>{eprintln!("Bad get 1. Trying to get {}, length {}.",n,occurrences.len()); return None;}
        };

        let start = occurence.end;
        
        let end = {
            if n<occurrences.len()-1
            {
                let next_occurence=match occurrences.get(n+1){
                    Some(o)=>o,
                    None=>{eprintln!("Bad get 2. Trying to get {}, length {}.",n+1,occurrences.len()); return None;}
                };
                next_occurence.start
            }
            else {
                cleaned_tokens.len()
            }
        };

        let mut subvec:Vec<String> = Vec::with_capacity(end-start);
        for n in start..end
        {
            let word = match cleaned_tokens.get(n)
            {
                Some(o)=>{
                    clean_tsword(o) //lower case, remove white space, and remove punctuation for comparisons below
                },
                None=>{eprintln!("Bad get 3 Trying to get {}, length {}.",n,cleaned_tokens.len()); return None;}
            };
            subvec.push(word);
        }

        println!("text_manip.rs: subvec {:?}",subvec);

        if subvec.contains(&HOUR.to_string()) || subvec.contains(&HOURS.to_string())
        {
            millis+=occurence.value*millis_per_hour;
        }
        else if subvec.contains(&MINUTE.to_string()) || subvec.contains(&MINUTES.to_string())
        {
            millis+=occurence.value*millis_per_minute;
        }
        else if subvec.contains(&SECOND.to_string()) || subvec.contains(&SECONDS.to_string())
        {
            millis+=occurence.value*millis_per_second;
        }
    }

    println!("text_manip.rs: millis={}",millis);
    let millis:u32=millis as u32;

    if millis>0
    {
        let retval=
        Some(Timer { 
            name: name,
            millis: millis,
            paused: false
        });
        println!("text_manip.rs: New timer: {:?}",retval);
        retval
    }
    else {
        println!("Millis<0, returning");
        None        
    }

    /*
    let mut continue_checking=num_sentence.len()>0;

    while continue_checking
    {
        for unit_to_check in possible_units
        {
            
        }
    }
    */
}