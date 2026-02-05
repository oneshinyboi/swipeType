# swipeType

Swipe typing engine and macOS application.

- **Rust engine** (`crates/swipe-engine`)
- [Web interface](https://swipetype.zimengxiong.com) (`apps/web`)
- **macOS app** (`apps/mac`)

The engine uses [Dynamic Time Warping (DTW)](https://en.wikipedia.org/wiki/Dynamic_time_warping) to measure similarity between the user's swipe path and pre-computed word paths on a QWERTY keyboard layout. Each word in the dictionary is converted into a series of coordinates based on key positions, and DTW computes a distance score by finding the optimal alignment between two sequences while allowing time warping. To handle 300k+ words efficiently, the engine applies a Sakoe-Chiba band window to constrain DTW to a diagonal band (O(n×w) instead of O(n×m)), maintains O(n) space by keeping only two rows of the matrix, and filters candidates by first/last character to avoid unnecessary comparisons.

## Installation

```bash
brew install zimengxiong/tools/swipetype
```

## Building

### Web

```bash
make build-website
make serve
```

### macOS App

```bash
make dmg-mac
make run
# Creates `apps/mac/build/SwipeType.dmg`
```

#### Screenshots

| Main                          | Help                          |
| ----------------------------- | ----------------------------- |
| ![Main](screenshots/main.png) | ![Help](screenshots/help.png) |

| Settings                              |
| ------------------------------------- |
| ![Settings](screenshots/settings.png) |
