//! Generates an SVG icon from the command line and writes it to stdout.
//!
//! Usage: `picturesque-tags [--size W[xH]] [--foreground] [TAG...]`

use std::process::ExitCode;

use picturesque_tags::{generate_icon_from_tags, generate_ptag_from_tags};

fn main() -> ExitCode {
    let mut tags: Vec<String> = Vec::new();
    let mut width = 100;
    let mut height = 100;
    let mut with_foreground = false;

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--foreground" | "-f" => with_foreground = true,
            "--help" | "-h" => {
                print_usage();
                return ExitCode::SUCCESS;
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

    let svg = if with_foreground {
        generate_icon_from_tags(&tags, width, height)
    } else {
        generate_ptag_from_tags(&tags, width, height)
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
    eprintln!("usage: picturesque-tags [--size W[xH]] [--foreground] [TAG...]");
    eprintln!("  TAG               tag string the icon is generated from (default: A)");
    eprintln!("                    two or more tags are sorted and up to four are composed in a 2x2 grid");
    eprintln!("  --size, -s W[xH]  icon width and height (default: 100)");
    eprintln!("  --foreground, -f  add a hand-drawn style foreground curve");
}
