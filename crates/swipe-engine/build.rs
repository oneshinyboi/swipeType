use std::collections::{HashMap, HashSet};
use std::{env, fs};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use swipe_types::types::BigramModel;
use bincode;
use bincode::config;
use codes_iso_639::part_1::LanguageCode;

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let out_dir = env::var("OUT_DIR").unwrap();

    let corpus_bin_dir = Path::new(&out_dir).join("corpuses/bin");
    if !corpus_bin_dir.exists() {
        fs::create_dir_all(&corpus_bin_dir).unwrap()
    }

    println!("cargo:rerun-if-changed=corpuses/plaintext");

    let corpus_text_dir = Path::new(&manifest_dir).join("corpuses/plaintext");
    if !corpus_text_dir.exists() {
        fs::create_dir_all(&corpus_text_dir).unwrap()
    }

    let language_dirs = fs::read_dir(corpus_text_dir).unwrap();

    for dir in language_dirs {
        let mut valid_words: HashSet<String> = HashSet::new();
        let dir = dir.unwrap();
        let dir_path = &dir.path();
        if let Ok(_lang_code) = dir_path.file_name().unwrap().to_str().unwrap().parse::<LanguageCode>() {

            let full_dest_file_name = format!("{}.bin", dir.file_name().to_str().unwrap());
            let dest_path = corpus_bin_dir.join(&full_dest_file_name);

            if env::var_os("CARGO_FORCE_CORPUS").is_some() || !dest_path.exists() {
                let mut word_list_path: Option<PathBuf> = None;
                let mut corpus_path: Option<PathBuf> = None;

                for potential_file in fs::read_dir(&dir_path).unwrap() {
                    let file_path = potential_file.unwrap().path();
                    if file_path.is_dir() { continue; }

                    let file_name = file_path.file_stem().unwrap().to_str().unwrap();
                    if file_name.contains("word_list") {
                        word_list_path = Some(file_path);
                    } else if file_name.contains("corpus") {
                        corpus_path = Some(file_path);
                    }
                }

                if let (Some(word_list_path), Some(corpus_path)) = (word_list_path, corpus_path) {
                    let file_name = word_list_path.file_stem().unwrap().to_str().unwrap();
                    if file_name.contains("word_list") {
                        let word_list = fs::read_to_string(word_list_path).unwrap();
                        for line in word_list.lines() {
                            valid_words.insert(String::from(line.trim()));
                        }
                    }
                    let file_name = corpus_path.file_stem().unwrap().to_str().unwrap();
                    if file_name.contains("corpus") {



                        let corpus_file = File::open(corpus_path).unwrap();
                        let corpus_reader = BufReader::new(corpus_file);

                        let model = create_bigram_model(corpus_reader, valid_words);
                        let serialized_model = bincode::encode_to_vec(&model, config::standard()).unwrap();
                        fs::write(&dest_path, serialized_model).expect(&format!("Failed to write {full_dest_file_name}"));
                        println!("cargo:warning=Processing corpus for {full_dest_file_name} complete");
                    }
                }
            }
        }
    }
}


fn create_bigram_model(corpus_reader: BufReader<File>, valid_words: HashSet<String>) -> BigramModel
{
    let mut pair_counts: HashMap<String, HashMap<String, u32>>= HashMap::new();
    let mut word_count: HashMap<String, u32> = HashMap::new();
    let mut freq= HashMap::new();
    let mut max_word_count: u32 = 0;

    for line in corpus_reader.lines() {

        let words: Vec<String> = line.unwrap()
            .split_whitespace()
            .map(|s| s.to_lowercase())
            .collect();

        //count unigrams
        for word in &words {
            if valid_words.contains(word) {
                let count = word_count.entry(word.clone()).or_default();
                *count += 1;
                max_word_count = max_word_count.max(*count)
            }
        }

        //count bigrams
        for window in words.windows(2) {
            let word1 = &window[0];
            let word2 = &window[1];

            if valid_words.contains(word1) && valid_words.contains(word2) {
                let inner_map = pair_counts.entry(word1.clone()).or_default();
                *inner_map.entry(word2.clone()).or_insert(0) += 1;
            }

        }
    }

    let max_word_count_float = max_word_count as f64;
    for map in &word_count {
        let float_count = *map.1 as f64;
        let log_freq = (float_count.ln() - 1.0) / max_word_count_float.ln();
        freq.insert(map.0.clone(), log_freq.max(0.0));
    }

    BigramModel {
        pair_counts,
        freq,
        words: valid_words.iter().cloned().collect(),
    }
}