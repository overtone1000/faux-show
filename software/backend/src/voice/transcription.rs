use serde::{Deserialize, Serialize};

#[derive(Serialize,Deserialize,Debug)]
pub struct TranscribedWord
{
    word:String,
    probability:f32
}

pub type Transcription = Vec<TranscribedWord>;