//! SVG definitions of the hatch patterns used to fill shapes.

use crate::colors::Color;

/// Kind of hatch pattern. `Fill` means a solid fill.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HatchPatternType {
    DownwardDiagonal,
    DownwardDiagonalReverse,
    UpwardDiagonal,
    UpwardDiagonalThick,
    Horizontal,
    Vertical,
    Cross,
    DiagonalCross,
    Dots,
    PolkaDots,
    Checker,
    Fill,
}

impl HatchPatternType {
    pub const ALL: [HatchPatternType; 12] = [
        HatchPatternType::DownwardDiagonal,
        HatchPatternType::DownwardDiagonalReverse,
        HatchPatternType::UpwardDiagonal,
        HatchPatternType::UpwardDiagonalThick,
        HatchPatternType::Horizontal,
        HatchPatternType::Vertical,
        HatchPatternType::Cross,
        HatchPatternType::DiagonalCross,
        HatchPatternType::Dots,
        HatchPatternType::PolkaDots,
        HatchPatternType::Checker,
        HatchPatternType::Fill,
    ];
    pub const COUNT: u32 = Self::ALL.len() as u32;

    pub fn from_index(index: u32) -> Self {
        Self::ALL[(index % Self::COUNT) as usize]
    }

    /// Returns the `<pattern>` element as an SVG string. Returns `None` for `Fill`.
    ///
    /// `id` must be unique within the document.
    pub fn to_svg_pattern(self, id: &str, color: Color) -> Option<String> {
        let hex = color.to_hex();
        let line = |x1: i32, y1: i32, x2: i32, y2: i32, width: i32| {
            format!(
                r#"<line x1="{x1}" y1="{y1}" x2="{x2}" y2="{y2}" stroke="{hex}" stroke-width="{width}"/>"#
            )
        };
        let circle = |cx: i32, cy: i32, r: i32| {
            format!(r#"<circle cx="{cx}" cy="{cy}" r="{r}" fill="{hex}"/>"#)
        };
        let rect = |x: i32, y: i32, w: i32, h: i32| {
            format!(r#"<rect x="{x}" y="{y}" width="{w}" height="{h}" fill="{hex}"/>"#)
        };

        // A diagonal line drawn directly always breaks at the tile boundary.
        // Rotating a vertical-line pattern by 45 degrees avoids that.
        let (size, rotate, children): (i32, Option<i32>, Vec<String>) = match self {
            HatchPatternType::DownwardDiagonal => (10, Some(-45), vec![line(5, 0, 5, 10, 3)]),
            HatchPatternType::DownwardDiagonalReverse => (
                10,
                Some(-45),
                vec![line(2, 0, 2, 10, 5), line(8, 0, 8, 10, 3)],
            ),
            HatchPatternType::UpwardDiagonal => (10, Some(45), vec![line(5, 0, 5, 10, 2)]),
            HatchPatternType::UpwardDiagonalThick => (10, Some(45), vec![line(5, 0, 5, 10, 5)]),
            HatchPatternType::Horizontal => (10, None, vec![line(0, 5, 10, 5, 3)]),
            HatchPatternType::Vertical => (10, None, vec![line(5, 0, 5, 10, 3)]),
            HatchPatternType::Cross => (10, None, vec![line(0, 5, 10, 5, 3), line(5, 0, 5, 10, 3)]),
            HatchPatternType::DiagonalCross => (
                10,
                None,
                vec![line(0, 0, 10, 10, 3), line(0, 10, 10, 0, 3)],
            ),
            HatchPatternType::Dots => (7, None, vec![line(1, 0, 1, 2, 2)]),
            HatchPatternType::PolkaDots => (20, None, vec![circle(4, 4, 4), circle(14, 14, 4)]),
            HatchPatternType::Checker => (20, None, vec![rect(0, 0, 10, 10), rect(10, 10, 10, 10)]),
            HatchPatternType::Fill => return None,
        };

        let transform = rotate
            .map(|deg| format!(r#" patternTransform="rotate({deg})""#))
            .unwrap_or_default();
        Some(format!(
            r#"<pattern id="{id}" width="{size}" height="{size}" patternUnits="userSpaceOnUse"{transform}>{}</pattern>"#,
            children.concat()
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fill_has_no_pattern() {
        assert_eq!(HatchPatternType::Fill.to_svg_pattern("p", Color::rgb(0, 0, 0)), None);
    }

    #[test]
    fn diagonal_pattern_is_rotated() {
        let svg = HatchPatternType::DownwardDiagonal
            .to_svg_pattern("pat1", Color::rgb(0xEB, 0xAE, 0x9B))
            .unwrap();
        assert!(svg.starts_with(r#"<pattern id="pat1" width="10" height="10""#));
        assert!(svg.contains(r#"patternTransform="rotate(-45)""#));
        assert!(svg.contains(r##"stroke="#EBAE9B""##));
    }

    #[test]
    fn horizontal_pattern_has_no_transform() {
        let svg = HatchPatternType::Horizontal
            .to_svg_pattern("p", Color::rgb(0, 0, 0))
            .unwrap();
        assert!(!svg.contains("patternTransform"));
    }

    #[test]
    fn every_non_fill_pattern_renders() {
        for t in HatchPatternType::ALL {
            if t != HatchPatternType::Fill {
                assert!(t.to_svg_pattern("p", Color::rgb(1, 2, 3)).is_some(), "{t:?}");
            }
        }
    }
}
