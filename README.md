# picturesque-tags (Rust)

Rust port of [sosuisen/picturesque-tags](https://github.com/sosuisen/picturesque-tags).
It generates SVG icons from tag strings. The same tag always produces the same background.

## Usage

### Library

```rust
use picturesque_tags::{generate_icon, generate_ptag};

// Background only
let svg = generate_ptag("apple", 100, 100);

// Background plus a hand-drawn style foreground curve
let svg = generate_icon("apple", 128, 128);

// Multiple tags: sorted, then up to four are composed in a 2x2 grid
use picturesque_tags::generate_icon_from_tags;
let svg = generate_icon_from_tags(&["cardbox", "card"], 128, 128);
```

Use `IconBuilder` directly for finer control.

```rust
use picturesque_tags::{Color, HatchPatternType, IconBuilder};

let mut builder = IconBuilder::new(100, 100);
builder
    .generate_bg_from_text("apple")
    .add_circle(HatchPatternType::Dots, Color::rgb(0x9B, 0xA9, 0xEB), 20.0, 0.0, 0.0)
    .add_foreground();
let svg = builder.to_svg();
```

### Command line

```sh
cargo run -- --size 128 apple > apple.svg
cargo run -- --size 128x96 --foreground apple > apple.svg
# Multiple tags are sorted and up to four are composed in a 2x2 grid
cargo run -- --size 128 cardbox card > cardbox_card.svg
```

### Gallery

Generates an HTML page with the same tag list as the original Blazor demo.

```sh
cargo run --example gallery > gallery.html
```

## Tests

```sh
cargo test
```

## Differences from the original

- The SVG is built as a string without an external SVG library.
- Pattern IDs are sequential numbers instead of GUIDs, so the same tag yields the same SVG string.
- Colors are always written as `#RRGGBB`.
- Characters outside the BMP count as one character (Rust `char`). The original counts them as two UTF-16 units.
