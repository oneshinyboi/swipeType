# swipe-engine

A fast swipe typing prediction engine written in Rust.

## Features

- **DTW Algorithm**: Uses Dynamic Time Warping for robust path comparison.
- **Fast**: Optimized with path simplification and early pruning.
- **WASM Support**: Can be compiled to WebAssembly for use in browsers.
- **FFI Support**: Can be used from other languages (like Swift for macOS apps).
- **Customizable**: Adjustable popularity weighting for word scoring.

## Usage

```rust
use swipe_engine::SwipeEngine;

let mut engine = SwipeEngine::new();
// Load a dictionary in the format "word	count"
engine.load_dictionary("hello	1000
world	500
");

let predictions = engine.predict("hello", 5);
for prediction in predictions {
    println!("{}: score={}", prediction.word, prediction.score);
}
```

## License

MIT
