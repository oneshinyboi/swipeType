//! Swipe typing prediction engine

pub mod dtw;
pub mod keyboard;


use bincode;
use codes_iso_639::part_1::LanguageCode;
use dtw::dtw_distance_fast;
use keyboard::{euclidean_dist, get_keyboard_layout, get_word_path, simplify_path};
use std::collections::HashMap;
use std::path::Path;
use std::{env, fs};
use swipe_types::types::{Dictionary, Point, Prediction};

pub use dtw::{dtw_distance, dtw_distance_fast as dtw_fast};
pub use keyboard::{
    euclidean_dist as euclidean_distance, get_keyboard_layout as keyboard_layout,
    get_word_path as word_path, simplify_path as path_simplify,
};
pub use swipe_types::types::Point as PointType;

/// Uses a Dynamic Time Warping (DTW) algorithm to compare swipe paths
/// against a dictionary of words.
pub struct SwipeEngine {
    dictionary: Dictionary,
    layout: HashMap<char, Point>,
    pop_weight: f64,
    bigram_weight: f64,

    by_first_letter: HashMap<char, Vec<usize>>,
    word_paths: Vec<Vec<Point>>,
}

impl SwipeEngine {
    pub fn new(lang_code: LanguageCode, layout: Option<HashMap<char, Point>>) -> Result<Self, String> {
        let lang = lang_code.to_string();
        let out_dir_string = env::var("DICT_PATH").unwrap();
        let out_dir = Path::new(&out_dir_string);

        let dict_path = out_dir.join(format!("{lang}.bin"));

        if !dict_path.exists() {
            return Err(format!(
                "Language {lang} is unsupported or dictionary file not found at {}",
                dict_path.display()
            ));
        }

        match fs::read(dict_path) {
            Ok(bytes) => {
                match bincode::decode_from_slice(&bytes, bincode::config::standard()) {
                    Ok((model, _len)) => {
                        let mut engine = Self {
                            dictionary: model,
                            layout: layout.unwrap_or_else(get_keyboard_layout),
                            pop_weight: 0.25,
                            bigram_weight: 0.5,
                            by_first_letter: HashMap::new(),
                            word_paths: Vec::new(),
                        };
                        engine.build_index();
                        Ok(engine)
                    }
                    Err(e) => Err(format!("Failed to decode dictionary for {}: {}", lang, e)),
                }
            }
            Err(e) => Err(format!(
                "Failed to read dictionary file for {}: {}",
                lang, e
            )),
        }
    }

    /// Higher values favor common words more heavily in the scoring function.
    pub fn set_pop_weight(&mut self, weight: f64) {
        self.pop_weight = weight;
    }

    /// Higher values favor words that are more likely to follow the previous word.
    pub fn set_bigram_weight(&mut self, weight: f64) {
        self.bigram_weight = weight;
    }

    fn build_index(&mut self) {
        self.by_first_letter.clear();
        self.word_paths.clear();
        self.word_paths.reserve(self.dictionary.words.len());
        for (idx, word) in self.dictionary.words.iter().enumerate() {
            if let Some(first) = word.chars().next() {
                self.by_first_letter
                    .entry(first.to_ascii_lowercase())
                    .or_insert_with(Vec::new)
                    .push(idx);
            }
            let raw_path = get_word_path(word, &self.layout);
            self.word_paths.push(simplify_path(&raw_path));
        }
    }

    pub fn word_count(&self) -> usize {
        self.dictionary.words.len()
    }

    /// Input string should be the sequence of characters the swipe path passes through.
    /// Returns predictions sorted by score.
    /// previous_word will be ignored if lib was compiled without use-pair-counts feature
    pub fn predict(&self, swipe_input: &str, previous_word: Option<&str>, limit: usize) -> Vec<Prediction> {
        let raw_input_path = get_word_path(swipe_input, &self.layout);
        if raw_input_path.is_empty() {
            return vec![];
        }

        let input_path = simplify_path(&raw_input_path);
        let input_len = input_path.len() as f64;

        let first_char = match swipe_input.chars().next() {
            Some(c) => c.to_ascii_lowercase(),
            None => return vec![],
        };
        let last_char = swipe_input.chars().last().unwrap().to_ascii_lowercase();
        let last_char_pt = self
            .layout
            .get(&last_char)
            .cloned()
            .unwrap_or(Point { x: 0.0, y: 0.0 });

        let candidate_indices = match self.by_first_letter.get(&first_char) {
            Some(indices) => indices,
            None => return vec![],
        };

        let window = (input_path.len() / 2).max(10);
        let mut best_score = f64::INFINITY;

        let mut candidates: Vec<(String, f64, f64, f64)> = candidate_indices
            .iter()
            .filter_map(|&idx| {
                let w = &self.dictionary.words[idx];

                let word_last_char = w.chars().last().unwrap().to_lowercase().next().unwrap();
                let mut end_penalty = 0.0;
                if word_last_char != last_char {
                    if let Some(word_last_pt) = self.layout.get(&word_last_char) {
                        end_penalty = euclidean_dist(&last_char_pt, word_last_pt) * 5.0;
                    } else {
                        end_penalty = 50.0;
                    }
                }

                let cutoff = best_score * input_len;
                let word_path = &self.word_paths[idx];
                let dist = dtw_distance_fast(&input_path, word_path, window, cutoff);

                if dist == f64::INFINITY {
                    return None;
                }

                let score = (dist + end_penalty) / input_len;
                if score < best_score {
                    best_score = score;
                }

                let word_info = self.dictionary.word_info.get(&w.as_str().to_lowercase());
                let mut word_freq = 0.0;
                let mut bigram_probability: f64 = 0.0;

                if let Some(word_info) = word_info {
                    word_freq = word_info.log_freq;
                    if let Some(previous_word) = previous_word {
                        let previous_word_lowercase = previous_word.to_lowercase();
                        if let Some(pair_counts) = &self.dictionary.pair_counts {
                            if let Some(pair_count_map) = pair_counts.get(&previous_word_lowercase) {
                                let bigram_count = pair_count_map.get(&w.as_str().to_lowercase()).unwrap_or(&0u32);
                                bigram_probability = (*bigram_count as f64) / (word_info.count as f64);
                            }
                        }
                    }
                }

                Some((w.clone(), score, word_freq, bigram_probability))
            })
            .collect();

        candidates.sort_by(|a, b| {
            let combined_a = a.1 - a.2 * self.pop_weight - a.3 * self.bigram_weight;
            let combined_b = b.1 - b.2 * self.pop_weight - b.3 * self.bigram_weight;
            //println!("{}, {}", a.3, b.3);
            combined_a
                .partial_cmp(&combined_b)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        candidates
            .into_iter()
            .take(limit)
            .map(|(word, score, freq, bigram_prob)| {
                let mut return_bigram_prob = None;
                if bigram_prob != 0.0 {
                    return_bigram_prob = Some(bigram_prob);
                }
                Prediction { word, score, freq, bigram_prob: return_bigram_prob}
            })
            .collect()
    }
}

impl Default for SwipeEngine {
    fn default() -> Self {
        Self::new(LanguageCode::En, None).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_creation() {
        let engine = SwipeEngine::new(LanguageCode::En, None).unwrap();
        assert!(engine.word_count() > 0, "Dictionary should be loaded by default");
    }


    #[test]
    fn test_prediction() {
        let engine = SwipeEngine::new(LanguageCode::En, None).unwrap();

        let predictions = engine.predict("mhgfcxsazxcvbnhytfdsasdftgfdsasdfgbnjmn", Some("to"), 5);
        println!("{:?}", predictions);
        assert!(!predictions.is_empty());

        let predictions = engine.predict("mhgfcxsazxcvbnhytfdsasdftgfdsasdfgbnjmn", None, 5);
        println!("{:?}", predictions);
        assert!(!predictions.is_empty());
    }
}
