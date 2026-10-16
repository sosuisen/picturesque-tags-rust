//! Generates a gallery HTML page from the same tag list as the original Blazor demo (Home.razor).
//!
//! Run: `cargo run --example gallery > gallery.html`

use picturesque_tags::{generate_icon, generate_icon_from_tags};

const TAGS: &[&str] = &[
    "apple", "banana", "cherry", "date", "elderberry", "fig", "grape", "honeydew", "kiwi", "lemon",
    "mango", "nectarine", "orange", "pear", "quince", "raspberry", "strawberry", "tangerine", "ugli",
    "vanilla", "watermelon", "xigua", "yuzu", "zucchini", "Avocado", "Blackberry", "Cantaloupe",
    "Dragonfruit", "Eggplant", "Figs", "Grapes", "Honeydew", "Iceberg", "Jicama", "Kiwi", "Lemon",
    "Mango", "Nectarine", "Orange", "Peach", "Quince", "Raspberry", "Strawberry", "Tangerine",
    "Ugli", "Vanilla", "Watermelon", "Xigua", "Yuzu", "Zucchini",
    "フリーライティング", "フリーライティング・朝", "フリークライミング", "cardbox",
];

// Tag combinations from Cards.cs in the original repository. Used to check multi-tag composition.
const TAG_SETS: &[&[&str]] = &[
    &["フリークライミング", "card"],
    &["cardbox", "card"],
    &["cardbox", "card", "framework"],
    &["zucchini", "yuzu", "xigua", "watermelon", "vanilla"],
    &[],
];

fn main() {
    let size = 128;
    println!("<!DOCTYPE html>");
    println!("<html><head><meta charset=\"utf-8\"><title>Picturesque Tags</title></head><body>");
    println!("<h1>Picturesque Tags</h1>");
    println!("<div style=\"display:flex;flex-wrap:wrap;\">");
    for tag in TAGS {
        let svg = generate_icon(tag, size, size);
        println!(
            "<figure style=\"margin:4px;\">{svg}<figcaption style=\"text-align:center;\">{tag}</figcaption></figure>"
        );
    }
    println!("</div>");
    println!("<h2>Multiple tags</h2>");
    println!("<div style=\"display:flex;flex-wrap:wrap;\">");
    for tags in TAG_SETS {
        let svg = generate_icon_from_tags(tags, size, size);
        let caption = if tags.is_empty() { "(none)".to_string() } else { tags.join(", ") };
        println!(
            "<figure style=\"margin:4px;\">{svg}<figcaption style=\"text-align:center;\">{caption}</figcaption></figure>"
        );
    }
    println!("</div></body></html>");
}
