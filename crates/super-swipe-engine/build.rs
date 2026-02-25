use std::collections::{HashMap, HashSet};
use std::{env, fs};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use swipe_types::types::{Dictionary, WordInfo};
use bincode;
use bincode::config;
use codes_iso_639::part_1::LanguageCode;

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let lang_data_bin_dir = Path::new(&manifest_dir).join("assets");

    if !lang_data_bin_dir.exists() {
        fs::create_dir_all(&lang_data_bin_dir).unwrap()
    }
    let
        lang_data_text_dir = match env::var("LANGDATA_DIR") {
        Ok(val) => PathBuf::from(val),
        Err(_) => Path::new(&manifest_dir).join("lang-data/plaintext"),
    };
    println!("cargo:rerun-if-changed={}", lang_data_text_dir.display());

    if !lang_data_text_dir.exists() {
        println!("cargo:warning=Please create the dir: {}", lang_data_text_dir.display());
        return
    }

    let language_dirs = fs::read_dir(lang_data_text_dir).unwrap();

    for dir in language_dirs {

        let dir = dir.unwrap();
        let dir_path = &dir.path();
        if let Ok(_lang_code) = dir_path.file_name().unwrap().to_str().unwrap().parse::<LanguageCode>() {

            let full_dest_file_name = format!("{}.bin", dir.file_name().to_str().unwrap());
            let dest_path = lang_data_bin_dir.join(&full_dest_file_name);

            if env::var_os("CARGO_FORCE_CORPUS").is_some() || !dest_path.exists() {
                let mut word_list_path: Option<PathBuf> = None;
                let mut corpus_path: Option<PathBuf> = None;
                let mut word_freq_path: Option<PathBuf> = None;

                for potential_file in fs::read_dir(&dir_path).unwrap() {
                    let file_path = potential_file.unwrap().path();
                    if file_path.is_dir() { continue; }

                    let file_name = file_path.file_stem().unwrap().to_str().unwrap();
                    if file_name.contains("word_list") {
                        word_list_path = Some(file_path);
                    } else if file_name.contains("corpus") {
                        corpus_path = Some(file_path);
                    } else if file_name.contains("word_freq") {
                        word_freq_path = Some(file_path)
                    }
                }
                if !env::var_os("CARGO_IGNORE_WORD_FREQUENCY_FILES").is_some() {
                    if let Some(word_freq_path) = word_freq_path {
                        let mut valid_words: HashSet<String> = HashSet::new();
                        let mut word_counts: HashMap<String, u32>  = HashMap::new();
                        let mut freq = HashMap::new();
                        let mut max_count: u32 = 0;

                        let word_frequencies = fs::read_to_string(word_freq_path).unwrap();

                        for line in word_frequencies.lines() {
                            let parts: Vec<&str> = line.split_whitespace().collect();

                            let raw_count: u32 = parts[1].parse().unwrap_or(u32::MAX);
                            word_counts.insert(String::from(parts[0]), raw_count);
                            max_count = max_count.max(raw_count);
                            valid_words.insert(String::from(parts[0]));
                        }
                        for word in word_counts {
                            let log_freq = (word.1 as f64).ln() - 1.0 / (max_count as f64).ln();
                            freq.insert(word.0, WordInfo {log_freq, count: word.1});
                        }
                        let model = Dictionary {
                            pair_counts: None,
                            words: valid_words.iter().cloned().collect(),
                            word_info: freq
                        };
                        let serialized_model = bincode::encode_to_vec(&model, config::standard()).unwrap();
                        fs::write(&dest_path, serialized_model).expect(&format!("Failed to write {full_dest_file_name}"));
                    }

                }



                else if let (Some(word_list_path), Some(corpus_path)) = (word_list_path, corpus_path) {
                    let mut valid_words: HashSet<String> = HashSet::new();
                    let mut valid_words_lowercase: HashSet<String> = HashSet::new();

                    let word_list = fs::read_to_string(word_list_path).unwrap();
                    for line in word_list.lines() {
                        let word = String::from(line.trim());
                        valid_words.insert(word.clone());
                        valid_words_lowercase.insert(word.to_lowercase());
                    }
                    let file_name = corpus_path.file_stem().unwrap().to_str().unwrap();
                    if file_name.contains("corpus") {

                        let corpus_file = File::open(corpus_path).unwrap();
                        let corpus_reader = BufReader::new(corpus_file);

                        let model = create_dictionary_from_corpus(corpus_reader, valid_words, valid_words_lowercase);
                        let serialized_model = bincode::encode_to_vec(&model, config::standard()).unwrap();
                        fs::write(&dest_path, serialized_model).expect(&format!("Failed to write {full_dest_file_name}"));
                    }
                }
            }
        }
    }
}


fn create_dictionary_from_corpus(corpus_reader: BufReader<File>, valid_words: HashSet<String>, valid_words_lowercase: HashSet<String>) -> Dictionary
{
    let mut pair_counts: HashMap<String, HashMap<String, u32>> = HashMap::new();
    let mut word_count: HashMap<String, u32> = HashMap::new();
    let mut freq= HashMap::new();
    let mut max_word_count: u32 = 0;

    for line in corpus_reader.lines() {
        let lowercase_words: Vec<String> = line.unwrap()
            .split_whitespace()
            .map(|s| s.to_lowercase())
            .collect();

        //count unigrams
        for lowercase_word in &lowercase_words {
            if valid_words_lowercase.contains(lowercase_word) {
                let count = word_count.entry(lowercase_word.clone()).or_default();
                *count += 1;
                max_word_count = max_word_count.max(*count)
            }
        }

        if env::var_os("CARGO_USE_PAIR_COUNTS").is_some() {
            //count bigrams
            for window in lowercase_words.windows(2) {
                let word1 = &window[0];
                let word2 = &window[1];

                if valid_words_lowercase.contains(word1) && valid_words_lowercase.contains(word2) {
                    let inner_map = pair_counts.entry(word1.clone()).or_default();
                    *inner_map.entry(word2.clone()).or_insert(0) += 1;
                }

            }
        }

    }

    let max_word_count_float = max_word_count as f64;
    for (string, word_count) in &word_count {
        let float_count = *word_count as f64;
        let log_freq = (float_count.ln() - 1.0) / max_word_count_float.ln();
        freq.insert(string.clone(), WordInfo {count: *word_count, log_freq: log_freq.max(0.0)});
    }

    let mut return_pair_count = None;
    if !pair_counts.is_empty() {
        return_pair_count = Some(pair_counts);
    }

    Dictionary {
        pair_counts: return_pair_count,
        word_info: freq,
        words: valid_words.iter().cloned().collect(),
    }
}
