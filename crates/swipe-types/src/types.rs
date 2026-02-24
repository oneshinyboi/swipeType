use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use bincode::{Decode, Encode};

#[derive(Clone, Copy, Debug)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Prediction {
    pub word: String,
    pub score: f64,
    pub freq: f64,
    pub bigram_prob: f64,
}


#[derive(Encode, Decode)]
pub struct BigramModel {
    pub pair_counts: HashMap<String, HashMap<String, u32>>,

    pub words: Vec<String>,
    pub word_info: HashMap<String, WordInfo>,
}

#[derive(Encode, Decode)]
pub struct WordInfo {
    pub log_freq: f64,
    pub count: u32,
}
impl Default for WordInfo {
    fn default() -> Self {
        Self {
            log_freq: 0.0,
            count: 0
        }
    }
}


impl BigramModel {
    pub fn new() -> Self {
        Self {
            pair_counts: HashMap::new(),
            words: Vec::new(),
            word_info: HashMap::new(),
        }
    }
}

impl Default for BigramModel {
    fn default() -> Self {
        Self::new()
    }
}
