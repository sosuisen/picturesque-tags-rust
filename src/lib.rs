//! Picturesque Tags: generate SVG icons from tag strings deterministically.
//!
//! Rust port of <https://github.com/sosuisen/picturesque-tags>.
//!
//! The background is generated from the tag. The foreground curve is generated
//! from a separate string called the signature, which stands for the owner's
//! handwritten sign. The same inputs always give the same SVG.

pub mod area;
pub mod colors;
pub mod geometry;
pub mod hatch_pattern;
pub mod icon_builder;

pub use area::AreaName;
pub use colors::{Color, ColorName};
pub use geometry::{Geometry, GeometryType, ShapeType};
pub use hatch_pattern::HatchPatternType;
pub use icon_builder::{ForegroundStyle, IconBuilder};

/// Maximum number of tags in a composed icon.
/// Tiles go to the top-left, top-right, bottom-left and bottom-right.
pub const MAX_COMPOSED_TAGS: usize = 4;

/// Signature and curve style for the foreground.
#[derive(Debug, Clone, Copy)]
struct Foreground<'a> {
    signature: &'a str,
    style: ForegroundStyle,
}

/// Generates an SVG icon with the background only.
///
/// Equivalent to `Program.GeneratePtag` in the original. No foreground curve.
pub fn generate_ptag(tag: &str, width: i32, height: i32) -> String {
    build_icon(tag, width, height, None).to_svg()
}

/// Generates an SVG icon with the background from `tag` and the foreground curve
/// from `signature`, drawn in the given `style`.
///
/// Equivalent to `Generator.GenerateIcon(int, int, string)` in the original,
/// except that the curve is deterministic.
pub fn generate_icon(
    tag: &str,
    signature: &str,
    style: ForegroundStyle,
    width: i32,
    height: i32,
) -> String {
    build_icon(tag, width, height, Some(Foreground { signature, style })).to_svg()
}

/// Generates an SVG icon with the foreground curve from a list of tags.
///
/// - No tags: returns the default icon.
/// - One tag: returns the icon for that tag.
/// - Two or more tags: sorts them, takes up to four, and composes them
///   into one icon in the order top-left, top-right, bottom-left, bottom-right.
///   The signature curve is drawn once, large, over the whole composed icon.
pub fn generate_icon_from_tags(
    tags: &[impl AsRef<str>],
    signature: &str,
    style: ForegroundStyle,
    width: i32,
    height: i32,
) -> String {
    generate_from_tags(tags, width, height, Some(Foreground { signature, style }))
}

/// Generates an SVG icon with the background only from a list of tags.
///
/// Behaves like `generate_icon_from_tags` with respect to the number of tags.
pub fn generate_ptag_from_tags(tags: &[impl AsRef<str>], width: i32, height: i32) -> String {
    generate_from_tags(tags, width, height, None)
}

fn generate_from_tags(
    tags: &[impl AsRef<str>],
    width: i32,
    height: i32,
    foreground: Option<Foreground<'_>>,
) -> String {
    match tags {
        [] => build_icon("", width, height, foreground).to_svg(),
        [tag] => build_icon(tag.as_ref(), width, height, foreground).to_svg(),
        _ => compose_icons(tags, width, height, foreground),
    }
}

/// Composes several tags into one icon.
///
/// Sorts the tags by Unicode code point and lays out up to four of them
/// as W/2 x H/2 tiles in a 2 x 2 grid.
/// Each tile is embedded as a nested `<svg>`, so shapes outside a tile are clipped.
/// The signature curve, if any, is drawn once over the whole icon, not per tile.
fn compose_icons(
    tags: &[impl AsRef<str>],
    width: i32,
    height: i32,
    foreground: Option<Foreground<'_>>,
) -> String {
    let mut sorted: Vec<&str> = tags.iter().map(AsRef::as_ref).collect();
    sorted.sort_unstable();

    let tile_width = width / 2;
    let tile_height = height / 2;
    let mut layers: Vec<String> = sorted
        .iter()
        .take(MAX_COMPOSED_TAGS)
        .enumerate()
        .map(|(i, tag)| {
            // 0: top-left, 1: top-right, 2: bottom-left, 3: bottom-right
            let x = (i % 2) as i32 * tile_width;
            let y = (i / 2) as i32 * tile_height;
            let tile = build_icon(tag, tile_width, tile_height, None).to_svg();
            // Duplicate pattern IDs in one document resolve to the first definition,
            // so prefix the IDs with the tile index.
            nest_svg(&tile, x, y)
                .replace(r#"id="pat"#, &format!(r#"id="t{i}pat"#))
                .replace("url(#pat", &format!("url(#t{i}pat"))
        })
        .collect();

    if let Some(fg) = foreground {
        let mut layer = IconBuilder::new(width, height);
        layer.add_foreground(fg.signature, fg.style);
        layers.push(nest_svg(&layer.to_svg(), 0, 0));
    }

    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}">{}</svg>"#,
        layers.concat()
    )
}

/// Turns a standalone SVG string into a nested `<svg>` placed at `(x, y)`.
fn nest_svg(svg: &str, x: i32, y: i32) -> String {
    svg.replacen("<svg ", &format!(r#"<svg x="{x}" y="{y}" "#), 1)
}

fn build_icon(tag: &str, width: i32, height: i32, foreground: Option<Foreground<'_>>) -> IconBuilder {
    let mut builder = IconBuilder::new(width, height);
    builder.generate_bg_from_text(tag);
    if let Some(fg) = foreground {
        builder.add_foreground(fg.signature, fg.style);
    }
    builder
}

#[cfg(test)]
mod tests {
    use super::*;

    const SIGN: &str = "Hidekazu";
    const STYLE: ForegroundStyle = ForegroundStyle::Crossing;

    #[test]
    fn single_tag_is_same_as_generate_ptag() {
        let from_tags = generate_ptag_from_tags(&["apple"], 100, 100);
        let direct = generate_ptag("apple", 100, 100);
        assert_eq!(from_tags, direct);
    }

    #[test]
    fn generate_icon_is_deterministic() {
        let a = generate_icon("apple", SIGN, STYLE, 100, 100);
        let b = generate_icon("apple", SIGN, STYLE, 100, 100);
        assert_eq!(a, b);
        assert!(a.contains("<path d=\"M "));
    }

    #[test]
    fn signature_changes_only_the_curve() {
        let a = generate_icon("apple", "alice", STYLE, 100, 100);
        let b = generate_icon("apple", "bob", STYLE, 100, 100);
        let background = |svg: &str| svg.split("<path ").next().unwrap().to_string();
        assert_eq!(background(&a), background(&b));
        assert_ne!(a, b);
    }

    #[test]
    fn style_changes_only_the_curve() {
        let a = generate_icon("apple", SIGN, ForegroundStyle::Crossing, 100, 100);
        let b = generate_icon("apple", SIGN, ForegroundStyle::NoCrossing, 100, 100);
        let background = |svg: &str| svg.split("<path ").next().unwrap().to_string();
        assert_eq!(background(&a), background(&b));
        assert_ne!(a, b);
    }

    #[test]
    fn no_tags_gives_default_icon() {
        let svg = generate_icon_from_tags(&[] as &[&str], SIGN, STYLE, 100, 100);
        assert!(svg.contains(r##"fill="#808080""##));
        assert!(svg.contains("<path d=\"M "));
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
    fn composed_icon_draws_one_large_curve_on_top() {
        let svg = generate_icon_from_tags(&["b", "a", "c", "d"], SIGN, STYLE, 100, 100);
        // Four tiles plus one full-size foreground layer.
        assert_eq!(svg.matches("<svg x=").count(), MAX_COMPOSED_TAGS + 1);
        assert_eq!(svg.matches("<path ").count(), 1);
        let curve_layer = svg.rfind("<svg x=").unwrap();
        assert!(svg[curve_layer..].starts_with(
            r#"<svg x="0" y="0" xmlns="http://www.w3.org/2000/svg" width="100" height="100""#
        ));
        assert!(svg[curve_layer..].contains("<path "));
        // The curve is the same one a single icon of the same size would get.
        let mut expected = IconBuilder::new(100, 100);
        expected.add_foreground(SIGN, STYLE);
        let expected_path = expected.to_svg();
        let expected_path = expected_path.split("<path ").nth(1).unwrap();
        assert!(svg.contains(expected_path));
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
