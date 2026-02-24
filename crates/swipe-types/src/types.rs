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
    pub bigram_prob: Option<f64>,
}


#[derive(Encode, Decode)]
pub struct Dictionary {
    pub pair_counts: Option<HashMap<String, HashMap<String, u32>>>, //all lowercase
    pub words: Vec<String>, // has uppercase proper representations
    pub word_info: HashMap<String, WordInfo>, // all lowercase
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


impl Dictionary {
    pub fn new() -> Self {
        Self {
            pair_counts: None,
            words: Vec::new(),
            word_info: HashMap::new(),
        }
    }
}

impl Default for Dictionary {
    fn default() -> Self {
        Self::new()
    }
}
