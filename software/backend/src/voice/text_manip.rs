use crate::voice::whisper_streaming::{LivekitTranscriptionSegment, LivekitTranscriptionWord};

pub fn clean_tsword(tsword:&LivekitTranscriptionWord)->String
{
    tsword.word.to_lowercase().trim().trim_matches(['?','!','.']).to_string()
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
    pub remaining_tokens:Vec<LivekitTranscriptionWord>
}

pub fn check_for_match_string(match_string:String, segments:&Vec<LivekitTranscriptionSegment>)->Option<MatchResult>
{
    let tidied_match_string_words:Vec<String>=clean_match_string_into_words(match_string);

    //println!("Tidied words: {:?}",tidied_words);

    let mut scores:Vec<f32>=Vec::new();
    
    let mut current_match_word_index=0;

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
                                                remaining_tokens
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

pub fn extract_timer_init(match_result:MatchResult)
{
    eprintln!("Need to extract timer info from token stream here");

    let possible_units = [
        "hour",
        "hours",
        "minute",
        "minutes",
        "second",
        "seconds"
    ];

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