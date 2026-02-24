# swipe-engine

A fast swipe typing prediction engine written in Rust.

## Features

- **DTW Algorithm**: Uses Dynamic Time Warping for robust path comparison.
- **Fast**: Optimized with path simplification and early pruning.
- **Customizable**: Adjustable popularity weighting for word scoring.

## Usage
In order to compile you must provide a `corpus.txt` and a `word_list.txt` inside your project directory as follows. To support multiple languages use more folders named with the 639-1 language code.

```plaintext
├── build.rs
├── Cargo.toml
├── corpuses
│   └── plaintext
│       └── en
│           ├── corpus.txt
│           └─ word_list.txt
├── LICENSE
├── README.md
└── src
    ├── rust1.rs
    └── rust2.rs
```
```rust
use swipe_engine::SwipeEngine;

let engine = SwipeEngine::new(LanguageCode::En, None).unwrap();

let predictions = engine.predict("hello", None, 5);
for prediction in predictions {
    println!("{}: score={}", prediction.word, prediction.score);
}
```

## License

MIT
