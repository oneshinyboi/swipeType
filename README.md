# super-swipe-engine

A fast swipe typing prediction engine written in Rust.

## Features

- **DTW Algorithm**: Uses Dynamic Time Warping for robust path comparison.
- **Fast**: Optimized with path simplification and early pruning.
- **Customizable**: Adjustable popularity weighting for word scoring.

## Compilation
In order to compile you must provide a `word_freq.txt` file or both a `corpus.txt` and a `word_list.txt` inside your project directory as follows. To support multiple languages use more folders named with the 639-1 language code.

```plaintext
.
├── assets
│   └── en.bin
├── build.rs
├── Cargo.toml
├── lang-data
│   └── plaintext
│       └── en
│           ├── corpus.txt
│           ├── word_freq.txt
│           └── word_list.txt
├── LICENSE
├── README.md
└── src
    ├── dtw.rs
    ├── keyboard.rs
    └── lib.rs
```
## Usage
```rust
use codes_iso_639::part_1::LanguageCode;
use super_swipe_engine::SwipeEngine;

let engine = SwipeEngine::new(LanguageCode::En, None).unwrap();

let predictions = engine.predict("hello", None, 5);
for prediction in predictions {
    println!("{}: score={}", prediction.word, prediction.score);
}
```

## License

MIT
