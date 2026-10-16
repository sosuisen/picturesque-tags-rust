//! Picturesque Tags: generate SVG icons from tag strings deterministically.
//!
//! Rust port of <https://github.com/sosuisen/picturesque-tags>.

pub mod area;
pub mod colors;
pub mod geometry;
pub mod hatch_pattern;
pub mod icon_builder;

pub use area::AreaName;
pub use colors::{Color, ColorName};
pub use geometry::{Geometry, GeometryType, ShapeType};
pub use hatch_pattern::HatchPatternType;
pub use icon_builder::IconBuilder;

/// Maximum number of tags in a composed icon.
/// Tiles go to the top-left, top-right, bottom-left and bottom-right.
pub const MAX_COMPOSED_TAGS: usize = 4;

/// Generates an SVG icon with the background only.
///
/// Equivalent to `Program.GeneratePtag` in the original. No foreground curve.
pub fn generate_ptag(tag: &str, width: i32, height: i32) -> String {
    build_icon(tag, width, height, false).to_svg()
}

/// Generates an SVG icon with the background and the foreground curve.
///
/// Equivalent to `Generator.GenerateIcon(int, int, string)` in the original.
pub fn generate_icon(tag: &str, width: i32, height: i32) -> String {
    build_icon(tag, width, height, true).to_svg()
}

/// Generates an SVG icon with the foreground curve from a list of tags.
///
/// - No tags: returns the default icon.
/// - One tag: returns the icon for that tag.
/// - Two or more tags: sorts them, takes up to four, and composes them
///   into one icon in the order top-left, top-right, bottom-left, bottom-right.
pub fn generate_icon_from_tags(tags: &[impl AsRef<str>], width: i32, height: i32) -> String {
    generate_from_tags(tags, width, height, true)
}

/// Generates an SVG icon with the background only from a list of tags.
///
/// Behaves like `generate_icon_from_tags` with respect to the number of tags.
pub fn generate_ptag_from_tags(tags: &[impl AsRef<str>], width: i32, height: i32) -> String {
    generate_from_tags(tags, width, height, false)
}

fn generate_from_tags(
    tags: &[impl AsRef<str>],
    width: i32,
    height: i32,
    with_foreground: bool,
) -> String {
    match tags {
        [] => {
            let mut builder = IconBuilder::new(width, height);
            builder.default_icon();
            builder.to_svg()
        }
        [tag] => build_icon(tag.as_ref(), width, height, with_foreground).to_svg(),
        _ => compose_icons(tags, width, height, with_foreground),
    }
}

/// Composes several tags into one icon.
///
/// Sorts the tags by Unicode code point and lays out up to four of them
/// as W/2 x H/2 tiles in a 2 x 2 grid.
/// Each tile is embedded as a nested `<svg>`, so shapes outside a tile are clipped.
fn compose_icons(
    tags: &[impl AsRef<str>],
    width: i32,
    height: i32,
    with_foreground: bool,
) -> String {
    let mut sorted: Vec<&str> = tags.iter().map(AsRef::as_ref).collect();
    sorted.sort_unstable();

    let tile_width = width / 2;
    let tile_height = height / 2;
    let tiles: String = sorted
        .iter()
        .take(MAX_COMPOSED_TAGS)
        .enumerate()
        .map(|(i, tag)| {
            // 0: top-left, 1: top-right, 2: bottom-left, 3: bottom-right
            let x = (i % 2) as i32 * tile_width;
            let y = (i / 2) as i32 * tile_height;
            build_icon(tag, tile_width, tile_height, with_foreground)
                .to_svg()
                .replacen("<svg ", &format!(r#"<svg x="{x}" y="{y}" "#), 1)
                // Duplicate pattern IDs in one document resolve to the first definition,
                // so prefix the IDs with the tile index.
                .replace(r#"id="pat"#, &format!(r#"id="t{i}pat"#))
                .replace("url(#pat", &format!("url(#t{i}pat"))
        })
        .collect();

    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}">{tiles}</svg>"#
    )
}

fn build_icon(tag: &str, width: i32, height: i32, with_foreground: bool) -> IconBuilder {
    let mut builder = IconBuilder::new(width, height);
    builder.generate_bg_from_text(tag);
    if with_foreground {
        builder.add_foreground();
    }
    builder
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_tag_is_same_as_generate_ptag() {
        let from_tags = generate_ptag_from_tags(&["apple"], 100, 100);
        let direct = generate_ptag("apple", 100, 100);
        assert_eq!(from_tags, direct);
    }

    #[test]
    fn no_tags_gives_default_icon() {
        let svg = generate_ptag_from_tags(&[] as &[&str], 100, 100);
        assert!(svg.contains(r##"fill="#808080""##));
    }

    #[test]
    fn multiple_tags_are_sorted_and_tiled() {
        let svg = generate_ptag_from_tags(&["banana", "apple"], 100, 80);
        assert!(svg.starts_with(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="80" viewBox="0 0 100 80">"#
        ));
        // After sorting, apple is top-left and banana is top-right.
        let apple_tile = svg.find(r#"<svg x="0" y="0" "#).expect("top-left tile");
        let banana_tile = svg.find(r#"<svg x="50" y="0" "#).expect("top-right tile");
        assert!(apple_tile < banana_tile);
        let apple_body = generate_ptag("apple", 50, 40)
            .replace(r#"id="pat"#, r#"id="t0pat"#)
            .replace("url(#pat", "url(#t0pat");
        let apple_body = apple_body.split_once('>').map(|(_, rest)| rest).unwrap();
        assert!(svg[apple_tile..banana_tile].contains(apple_body));
    }

    #[test]
    fn three_tags_leave_bottom_right_empty() {
        let svg = generate_ptag_from_tags(&["c", "a", "b"], 100, 100);
        assert!(svg.contains(r#"<svg x="0" y="50" "#));
        assert!(!svg.contains(r#"<svg x="50" y="50" "#));
    }

    #[test]
    fn more_than_four_tags_are_truncated() {
        let svg = generate_ptag_from_tags(&["e", "d", "c", "b", "a"], 100, 100);
        assert_eq!(svg.matches("<svg x=").count(), MAX_COMPOSED_TAGS);
    }

    #[test]
    fn composed_pattern_ids_do_not_collide() {
        let svg = generate_ptag_from_tags(&["abcdef", "ghijkl", "mnopqr", "stuvwx"], 200, 200);
        let ids: Vec<&str> = svg
            .match_indices("<pattern id=\"")
            .map(|(pos, m)| {
                let start = pos + m.len();
                let end = svg[start..].find('"').unwrap() + start;
                &svg[start..end]
            })
            .collect();
        let mut unique = ids.clone();
        unique.sort_unstable();
        unique.dedup();
        assert_eq!(ids.len(), unique.len());
        // Element and attribute names must stay untouched.
        assert!(!svg.contains("t0pattern"));
        assert!(svg.contains("<pattern id=\"t0pat0\""));
        // References must be rewritten to the same IDs.
        assert!(svg.contains("url(#t0pat0)"));
    }
}
