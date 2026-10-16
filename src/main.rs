//! Generates an SVG icon from the command line and writes it to stdout.
//!
//! Usage: `picturesque-tags [--size W[xH]] [--signature TEXT] [TAG...]`

use std::process::ExitCode;

use picturesque_tags::{generate_icon_from_tags, generate_ptag_from_tags};

fn main() -> ExitCode {
    let mut tags: Vec<String> = Vec::new();
    let mut width = 100;
    let mut height = 100;
    let mut signature: Option<String> = None;

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" | "-h" => {
                print_usage();
                return ExitCode::SUCCESS;
            }
            "--signature" | "-g" => {
                let Some(value) = args.next() else {
                    eprintln!("--signature requires a value");
                    print_usage();
                    return ExitCode::FAILURE;
                };
                signature = Some(value);
            }
            "--size" | "-s" => {
                let Some(value) = args.next() else {
                    eprintln!("--size requires a value");
                    print_usage();
                    return ExitCode::FAILURE;
                };
                match parse_size(&value) {
                    Some((w, h)) => {
                        width = w;
                        height = h;
                    }
                    None => {
                        eprintln!("size must be W or WxH with positive integers: {value}");
                        print_usage();
                        return ExitCode::FAILURE;
                    }
                }
            }
            _ => tags.push(arg),
        }
    }

    // No tags would produce the default icon, so use "A" like the original Program.Main.
    if tags.is_empty() {
        tags.push(String::from("A"));
    }

    let svg = match signature {
        Some(signature) => generate_icon_from_tags(&tags, &signature, width, height),
        None => generate_ptag_from_tags(&tags, width, height),
    };
    println!("{svg}");
    ExitCode::SUCCESS
}

/// Parses `W` or `WxH`.
fn parse_size(value: &str) -> Option<(i32, i32)> {
    let positive = |s: &str| s.parse::<i32>().ok().filter(|n| *n > 0);
    match value.split_once('x') {
        Some((w, h)) => Some((positive(w)?, positive(h)?)),
        None => {
            let w = positive(value)?;
            Some((w, w))
        }
    }
}

fn print_usage() {
    eprintln!("usage: picturesque-tags [--size W[xH]] [--signature TEXT] [TAG...]");
    eprintln!("  TAG                  tag string the background is generated from (default: A)");
    eprintln!("                       two or more tags are sorted and up to four are composed in a 2x2 grid");
    eprintln!("  --size, -s W[xH]     icon width and height (default: 100)");
    eprintln!("  --signature, -g TEXT draw a hand-drawn style curve generated from TEXT on top");
}
