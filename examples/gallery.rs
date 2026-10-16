//! Generates a gallery HTML page from the same tag list as the original Blazor demo (Home.razor).
//!
//! Run: `cargo run --example gallery > gallery.html`

use picturesque_tags::{generate_icon, generate_icon_from_tags, generate_ptag, generate_ptag_from_tags};

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

// A few examples with a signature curve. Each pair is (signature, tags).
const SIGNED: &[(&str, &[&str])] = &[
    ("Alice", &["apple"]),
    ("Bob", &["apple"]),
    ("Alice", &["cardbox", "card"]),
    ("Bob", &["cardbox", "card", "framework"]),
    ("Alice", &[]),
];

fn figure(svg: &str, caption: &str) {
    println!(
        "<figure style=\"margin:4px;\">{svg}<figcaption style=\"text-align:center;\">{caption}</figcaption></figure>"
    );
}

fn tags_caption(tags: &[&str]) -> String {
    if tags.is_empty() {
        "(none)".to_string()
    } else {
        tags.join(", ")
    }
}

fn main() {
    let size = 128;
    println!("<!DOCTYPE html>");
    println!("<html><head><meta charset=\"utf-8\"><title>Picturesque Tags</title></head><body>");
    println!("<h1>Picturesque Tags</h1>");

    println!("<h2>Single tag</h2>");
    println!("<div style=\"display:flex;flex-wrap:wrap;\">");
    for tag in TAGS {
        figure(&generate_ptag(tag, size, size), tag);
    }
    println!("</div>");

    println!("<h2>Multiple tags</h2>");
    println!("<div style=\"display:flex;flex-wrap:wrap;\">");
    for tags in TAG_SETS {
        figure(&generate_ptag_from_tags(tags, size, size), &tags_caption(tags));
    }
    println!("</div>");

    println!("<h2>With signature</h2>");
    println!("<div style=\"display:flex;flex-wrap:wrap;\">");
    for (signature, tags) in SIGNED {
        let svg = match tags {
            [tag] => generate_icon(tag, signature, size, size),
            _ => generate_icon_from_tags(tags, signature, size, size),
        };
        figure(&svg, &format!("{} / sign: {signature}", tags_caption(tags)));
    }
    println!("</div></body></html>");
}
