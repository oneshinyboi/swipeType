# swipe-engine

A fast swipe typing prediction engine written in Rust.

## Features

- **DTW Algorithm**: Uses Dynamic Time Warping for robust path comparison.
- **Fast**: Optimized with path simplification and early pruning.
- **FFI Support**: Can be used from other languages (like Swift for macOS apps).
- **Customizable**: Adjustable popularity weighting for word scoring.

## Usage

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
