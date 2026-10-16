# picturesque-tags (Rust)

Rust port of [sosuisen/picturesque-tags](https://github.com/sosuisen/picturesque-tags).
It generates SVG icons from tag strings. The background comes from the tag.
The hand-drawn style curve on top comes from a separate string called the signature,
which stands for the owner's handwritten sign. The same inputs always produce the same SVG.

## Usage

### Library

```rust
use picturesque_tags::{generate_icon, generate_ptag};

// Background only
let svg = generate_ptag("apple", 100, 100);

// Background plus a foreground curve generated from the signature
let svg = generate_icon("apple", "Hidekazu", 128, 128);

// Multiple tags: sorted, then up to four are composed in a 2x2 grid.
// The signature curve is drawn once over the whole icon.
use picturesque_tags::generate_icon_from_tags;
let svg = generate_icon_from_tags(&["cardbox", "card"], "Hidekazu", 128, 128);
```

Use `IconBuilder` directly for finer control.

```rust
use picturesque_tags::{Color, HatchPatternType, IconBuilder};

let mut builder = IconBuilder::new(100, 100);
builder
    .generate_bg_from_text("apple")
    .add_circle(HatchPatternType::Dots, Color::rgb(0x9B, 0xA9, 0xEB), 20.0, 0.0, 0.0)
    .add_foreground("Hidekazu");
let svg = builder.to_svg();
```

### Command line

```sh
cargo run -- --size 128 apple > apple.svg
cargo run -- --size 128x96 --signature Hidekazu apple > apple.svg
# Multiple tags are sorted and up to four are composed in a 2x2 grid
cargo run -- --size 128 --signature Hidekazu cardbox card > cardbox_card.svg
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

- The foreground curve is generated from the signature with a seeded PCG32 generator. The original used a non-deterministic random generator.
- The SVG is built as a string without an external SVG library.
- Pattern IDs are sequential numbers instead of GUIDs, so the same tag yields the same SVG string.
- Colors are always written as `#RRGGBB`.
- Characters outside the BMP count as one character (Rust `char`). The original counts them as two UTF-16 units.
