use std::collections::HashMap;
use std::{env, fs};
use std::hash::Hash;
use std::os::nuttx;
use std::path::Path;

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let corpus_text_dir = Path::new(&manifest_dir).join("corpuses/plaintext");

    let corpuses = fs::read_dir(corpus_text_dir).unwrap();

    for corpus in corpuses {
        let corpus = fs::read_to_string(corpus.unwrap().path()).unwrap();
        let pair_count = process_corpus(corpus);
    }

}
fn process_corpus(corpus: String) -> HashMap<String, HashMap<String, u32>> {
    let mut pair_counts: HashMap<String, HashMap<String, u32>> = HashMap::new();
    for line in corpus.lines() {

        let words: Vec<String> = line
            .split_whitespace()
            .map(|s| s.to_lowercase())
            .collect();

        for window in words.windows(2) {
            let word1 = &window[0];
            let word2 = &window[1];
            let inner_map = pair_counts.entry(word1.clone()).or_default();
            *inner_map.entry(word2.clone()).or_insert(0) += 1;
        }
    }
    pair_counts
}