//! Builder that assembles an SVG icon from a tag string.

use std::f64::consts::PI;

use rand::{RngExt, SeedableRng};
use rand_pcg::Pcg32;
use sha1::{Digest, Sha1};

use crate::area::AreaName;
use crate::colors::{Color, ColorName};
use crate::geometry::{Geometry, GeometryType, ShapeType};
use crate::hatch_pattern::HatchPatternType;

/// Higher harmonics k added to the foreground outline. Low frequencies keep the shape calm.
const FOREGROUND_HARMONICS: [u32; 4] = [2, 3, 4, 5];

/// Number of points sampled on the foreground outline.
/// The inner loop spans only part of the circle, so it needs more samples than a plain blob.
const FOREGROUND_SAMPLES: usize = 24;

/// How many times the harmonics are redrawn when they do not give exactly one loop.
const FOREGROUND_MAX_TRIES: usize = 16;

/// One sine term of the foreground outline: amplitude × sin(k × θ + phase).
#[derive(Debug, Clone, Copy)]
struct Harmonic {
    k: u32,
    amplitude: f32,
    phase: f32,
}

/// Shape family of the foreground curve.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ForegroundStyle {
    /// A closed curve that crosses itself exactly once, like a loop in a handwritten sign.
    #[default]
    Crossing,
    /// A closed curve that never crosses itself. It can still have dents.
    NoCrossing,
}

/// SVG icon builder.
///
/// The background shapes are chosen deterministically from the tag by `generate_bg_from_text`.
/// The foreground curve is chosen deterministically from the signature by `add_foreground`.
#[derive(Debug, Clone)]
pub struct IconBuilder {
    width: i32,
    height: i32,
    pattern_defs: Vec<String>,
    bg_group: Vec<String>,
    fg_group: Vec<String>,
}

impl IconBuilder {
    pub fn new(width: i32, height: i32) -> Self {
        Self {
            width,
            height,
            pattern_defs: Vec::new(),
            bg_group: Vec::new(),
            fg_group: Vec::new(),
        }
    }

    pub fn width(&self) -> i32 {
        self.width
    }

    pub fn height(&self) -> i32 {
        self.height
    }

    /// Returns whether the character is an ideogram (kanji).
    ///
    /// Hiragana and katakana are phonetic, so they return `false`.
    /// Katakana often carries meaning in fewer characters than the original
    /// English word, so returning `true` for it might be reasonable,
    /// but the original implementation returns `false`, and so does this one.
    fn is_ideogram(c: char) -> bool {
        const KANJI_RANGES: [(u32, u32); 3] = [(0x4E00, 0x9FFF), (0x3400, 0x4DBF), (0xF900, 0xFAFF)];
        let code = c as u32;
        KANJI_RANGES
            .iter()
            .any(|&(start, end)| (start..=end).contains(&code))
    }

    /// Reads the tag from the start and cuts a block each time the character
    /// weight reaches 3 or more.
    ///
    /// Ideograms weigh 2, every other character weighs 1.
    /// The fourth block takes all remaining characters.
    fn split_blocks(tag: &str) -> [String; 4] {
        let chars: Vec<char> = tag.chars().collect();
        let mut blocks: [String; 4] = Default::default();
        let mut filled = 0;
        let mut char_count = 0;
        let mut tmp = String::new();

        for (i, &c) in chars.iter().enumerate() {
            char_count += if Self::is_ideogram(c) { 2 } else { 1 };
            tmp.push(c);
            let is_last = i == chars.len() - 1;

            // The last block is not cut by weight. It takes everything left.
            let should_close = if filled < 3 {
                char_count >= 3 || is_last
            } else if filled == 3 {
                is_last
            } else {
                false
            };
            if should_close {
                blocks[filled] = std::mem::take(&mut tmp);
                filled += 1;
                char_count = 0;
            }
        }
        blocks
    }

    /// Returns the first 4 bytes of the SHA-1 hash of the string as a little-endian u32.
    ///
    /// .NET's `GetHashCode` is not stable across environments, so it is not used.
    fn hash_u32(s: &str) -> u32 {
        let digest = Sha1::digest(s.as_bytes());
        u32::from_le_bytes([digest[0], digest[1], digest[2], digest[3]])
    }

    /// Returns the first 8 bytes of the SHA-1 hash of the string as a little-endian u64.
    /// Used as the seed of the foreground random number generator.
    fn hash_u64(s: &str) -> u64 {
        let digest = Sha1::digest(s.as_bytes());
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&digest[..8]);
        u64::from_le_bytes(bytes)
    }

    /// Adds background shapes chosen deterministically from the tag string.
    ///
    /// An empty tag produces the default background (a gray circle).
    pub fn generate_bg_from_text(&mut self, tag: &str) -> &mut Self {
        if tag.is_empty() {
            return self.default_background();
        }

        let [block1, block2, block3, block4] = Self::split_blocks(tag);
        let hash1 = Self::hash_u32(&block1);
        let hash2 = Self::hash_u32(&block2);
        let hash3 = Self::hash_u32(&block3);
        let hash4 = Self::hash_u32(&block4);

        // Reduce hash1 modulo (ColorName x ShapeType x GeometryType), then split
        // it into quotients and remainders to get each index.
        let index1 = hash1 % (ColorName::COUNT * ShapeType::COUNT * GeometryType::COUNT);
        let main_color_index = index1 / (ShapeType::COUNT * GeometryType::COUNT);
        let remainder = index1 % (ShapeType::COUNT * GeometryType::COUNT);
        let main_shape_index = remainder / GeometryType::COUNT;
        let main_geometry_index = remainder % GeometryType::COUNT;

        let main_color = ColorName::from_index(main_color_index).color();
        let main_geom = Geometry::lookup(AreaName::BgMain, GeometryType::from_index(main_geometry_index));

        let main_hatch = if block2.is_empty() {
            HatchPatternType::Fill
        } else {
            HatchPatternType::from_index(hash2)
        };

        let width = self.width as f32;

        if !block3.is_empty() {
            let index3 = hash3 % (ColorName::COUNT * GeometryType::COUNT);
            let color_index = index3 / GeometryType::COUNT;
            let geometry_index = index3 % GeometryType::COUNT;
            let color = ColorName::from_index(color_index).color().change_brightness(0.9);
            let geom = Geometry::lookup(AreaName::BgSub1, GeometryType::from_index(geometry_index));
            self.add_circle(
                HatchPatternType::Fill,
                color,
                geom.radius * width,
                geom.center_x * width,
                geom.center_y * width,
            );
        }

        if !block4.is_empty() {
            let index4 = hash4 % (ColorName::COUNT * GeometryType::COUNT);
            let color_index = index4 / GeometryType::COUNT;
            let geometry_index = index4 % GeometryType::COUNT;
            let color = ColorName::from_index(color_index).color().change_brightness(1.1);
            let geom = Geometry::lookup(AreaName::BgSub2, GeometryType::from_index(geometry_index));
            self.add_circle(
                HatchPatternType::Fill,
                color,
                geom.radius * width,
                geom.center_x * width,
                geom.center_y * width,
            );
        }

        let radius = main_geom.radius * width;
        let cx = main_geom.center_x * width;
        let cy = main_geom.center_y * width;
        match ShapeType::from_index(main_shape_index) {
            ShapeType::Circle => self.add_circle(main_hatch, main_color, radius, cx, cy),
            ShapeType::Diamond => self.add_diamond(main_hatch, main_color, radius, cx, cy),
            ShapeType::Pentagon => self.add_pentagon(main_hatch, main_color, radius, cx, cy),
        };

        self
    }

    /// Registers the pattern and returns the value for the shape's `fill` attribute.
    fn register_fill(&mut self, hatch: HatchPatternType, color: Color) -> String {
        let id = format!("pat{}", self.pattern_defs.len());
        match hatch.to_svg_pattern(&id, color) {
            Some(pattern) => {
                self.pattern_defs.push(pattern);
                format!("url(#{id})")
            }
            None => color.to_hex(),
        }
    }

    pub fn add_circle(
        &mut self,
        hatch: HatchPatternType,
        color: Color,
        radius: f32,
        center_x: f32,
        center_y: f32,
    ) -> &mut Self {
        let fill = self.register_fill(hatch, color);
        self.bg_group.push(format!(
            r#"<circle cx="{center_x}" cy="{center_y}" r="{radius}" fill="{fill}"/>"#
        ));
        self
    }

    pub fn add_diamond(
        &mut self,
        hatch: HatchPatternType,
        color: Color,
        radius: f32,
        center_x: f32,
        center_y: f32,
    ) -> &mut Self {
        let points = [
            (center_x + radius, center_y),
            (center_x, center_y + radius),
            (center_x - radius, center_y),
            (center_x, center_y - radius),
        ];
        self.add_polygon(hatch, color, &points)
    }

    pub fn add_pentagon(
        &mut self,
        hatch: HatchPatternType,
        color: Color,
        radius: f32,
        center_x: f32,
        center_y: f32,
    ) -> &mut Self {
        let points: Vec<(f32, f32)> = (0..5)
            .map(|k| {
                let angle = 2.0 * k as f64 * PI / 5.0;
                (
                    center_x + (radius as f64 * angle.cos()) as f32,
                    center_y + (radius as f64 * angle.sin()) as f32,
                )
            })
            .collect();
        self.add_polygon(hatch, color, &points)
    }

    fn add_polygon(&mut self, hatch: HatchPatternType, color: Color, points: &[(f32, f32)]) -> &mut Self {
        let fill = self.register_fill(hatch, color);
        let points: Vec<String> = points.iter().map(|(x, y)| format!("{x},{y}")).collect();
        self.bg_group.push(format!(
            r#"<polygon points="{}" fill="{fill}"/>"#,
            points.join(" ")
        ));
        self
    }

    /// Adds a hand-drawn style closed curve to the foreground.
    ///
    /// The curve is chosen deterministically from `signature`, which stands for
    /// the owner's handwritten sign. The same signature always gives the same curve.
    ///
    /// The outline is a polar function r(θ) = R × (1 + Σ a_k × sin(kθ + φ_k)).
    ///
    /// With `ForegroundStyle::Crossing`, the k = 1 term has an amplitude above 1,
    /// which makes r negative on part of the circle. The curve then passes through
    /// the origin twice and crosses itself exactly once there (a limaçon).
    /// With `ForegroundStyle::NoCrossing`, only k = 2..5 are used and their total
    /// amplitude stays below 1, so r is always positive and the curve never crosses itself.
    /// The sampled points are joined with Catmull-Rom splines converted to cubic Beziers.
    pub fn add_foreground(&mut self, signature: &str, style: ForegroundStyle) -> &mut Self {
        // PCG32 is used instead of rand's default generators because its algorithm
        // is fixed, so the same signature keeps giving the same curve across versions.
        let mut rng = Pcg32::seed_from_u64(Self::hash_u64(signature));
        let half = self.width / 2;
        // Integer arithmetic keeps the radius an exact pixel count like the original.
        let r_int = half * rng.random_range(70..90) / 100;
        let base_radius = r_int as f32;

        // Jitter of the center. Use 0 when the range would be empty.
        let jitter_max = half - r_int;
        let mut jitter = || -> f32 {
            if jitter_max > 0 {
                rng.random_range(0..jitter_max) as f32
            } else {
                0.0
            }
        };
        let (cx, cy) = (jitter(), jitter());

        let harmonics = match style {
            ForegroundStyle::Crossing => Self::draw_single_loop_harmonics(&mut rng),
            ForegroundStyle::NoCrossing => Self::draw_no_crossing_harmonics(&mut rng),
        };

        let profile: Vec<f32> = (0..FOREGROUND_SAMPLES)
            .map(|i| Self::radial_profile(Self::sample_angle(i, FOREGROUND_SAMPLES), &harmonics))
            .collect();
        // The harmonics push the outline beyond R, so scale the largest reach back to R.
        let max_reach = profile.iter().fold(0.0_f32, |acc, m| acc.max(m.abs()));
        let scale = base_radius / max_reach;
        let raw: Vec<(f32, f32)> = profile
            .iter()
            .enumerate()
            .map(|(i, m)| {
                let theta = Self::sample_angle(i, FOREGROUND_SAMPLES);
                (m * scale * theta.cos(), m * scale * theta.sin())
            })
            .collect();

        // The outline is lopsided, so center its bounding box on the jittered center.
        let (min_x, max_x, min_y, max_y) = raw.iter().fold(
            (f32::MAX, f32::MIN, f32::MAX, f32::MIN),
            |(min_x, max_x, min_y, max_y), &(x, y)| {
                (min_x.min(x), max_x.max(x), min_y.min(y), max_y.max(y))
            },
        );
        let shift_x = cx - (min_x + max_x) / 2.0;
        let shift_y = cy - (min_y + max_y) / 2.0;
        let points: Vec<(f32, f32)> = raw.iter().map(|&(x, y)| (x + shift_x, y + shift_y)).collect();

        let d = Self::closed_catmull_rom_path(&points);
        self.fg_group.push(format!(
            r##"<path d="{d}" stroke="#000000" stroke-width="5" fill="none"/>"##
        ));
        self
    }

    fn sample_angle(i: usize, samples: usize) -> f32 {
        2.0 * (PI as f32) * i as f32 / samples as f32
    }

    /// Draws harmonics from the generator until they describe exactly one loop.
    ///
    /// The ranges almost always give one loop on the first try. The check exists
    /// because strong higher harmonics can add extra zero crossings in rare cases.
    fn draw_single_loop_harmonics(rng: &mut Pcg32) -> Vec<Harmonic> {
        for _ in 0..FOREGROUND_MAX_TRIES {
            let harmonics = Self::draw_harmonics_once(rng);
            if Self::has_single_loop(&harmonics) {
                return harmonics;
            }
        }
        // A plain limaçon always has exactly one loop.
        vec![Harmonic {
            k: 1,
            amplitude: 1.5,
            phase: 0.0,
        }]
    }

    fn draw_harmonics_once(rng: &mut Pcg32) -> Vec<Harmonic> {
        // Loop size is (c - 1) x R, so c in 1.4..1.8 gives a clearly visible loop.
        let loop_amplitude = rng.random_range(140..=180) as f32 / 100.0;
        let loop_phase = Self::draw_phase(rng);

        // Higher harmonics share a small budget so they cannot add extra zero crossings
        // (needs to stay well below loop_amplitude - 1) or antipodal crossings (needs < 1).
        let budget = rng.random_range(5..=20) as f32 / 100.0;
        let mut harmonics = vec![Harmonic {
            k: 1,
            amplitude: loop_amplitude,
            phase: loop_phase,
        }];
        harmonics.extend(Self::draw_higher_harmonics(rng, budget));
        harmonics
    }

    /// Draws harmonics for a curve that never crosses itself.
    ///
    /// Only k = 2..5 are used. Their total amplitude 0.15..0.45 stays below 1,
    /// so r(θ) is always positive and the curve is star-shaped around the origin.
    /// Dents appear where the higher harmonics pull inward.
    fn draw_no_crossing_harmonics(rng: &mut Pcg32) -> Vec<Harmonic> {
        let budget = rng.random_range(15..=45) as f32 / 100.0;
        Self::draw_higher_harmonics(rng, budget)
    }

    /// Splits `budget` over k = 2..5 by random weights and draws a phase for each.
    fn draw_higher_harmonics(rng: &mut Pcg32, budget: f32) -> Vec<Harmonic> {
        let weights: Vec<f32> = FOREGROUND_HARMONICS
            .iter()
            .map(|_| rng.random_range(1..=10) as f32)
            .collect();
        let weight_sum: f32 = weights.iter().sum();
        FOREGROUND_HARMONICS
            .iter()
            .zip(weights)
            .map(|(&k, w)| Harmonic {
                k,
                amplitude: budget * w / weight_sum,
                phase: Self::draw_phase(rng),
            })
            .collect()
    }

    fn draw_phase(rng: &mut Pcg32) -> f32 {
        rng.random_range(0..360) as f32 * (PI as f32) / 180.0
    }

    /// Returns whether the polar outline crosses itself exactly once.
    ///
    /// A polar curve can only meet itself at the origin (where r changes sign) or at
    /// antipodal angles where r(θ) = -r(θ + π). Exactly two sign changes give one
    /// crossing at the origin, and r(θ) + r(θ + π) > 0 everywhere rules out the rest.
    fn has_single_loop(harmonics: &[Harmonic]) -> bool {
        const CHECK_SAMPLES: usize = 360;
        let m: Vec<f32> = (0..CHECK_SAMPLES)
            .map(|i| Self::radial_profile(Self::sample_angle(i, CHECK_SAMPLES), harmonics))
            .collect();
        let sign_changes = (0..CHECK_SAMPLES)
            .filter(|&i| (m[i] < 0.0) != (m[(i + 1) % CHECK_SAMPLES] < 0.0))
            .count();
        let no_antipodal_crossing = (0..CHECK_SAMPLES / 2).all(|i| m[i] + m[i + CHECK_SAMPLES / 2] > 0.0);
        sign_changes == 2 && no_antipodal_crossing
    }

    /// Returns the radius multiplier 1 + Σ a_k × sin(kθ + φ_k).
    fn radial_profile(theta: f32, harmonics: &[Harmonic]) -> f32 {
        1.0 + harmonics
            .iter()
            .map(|h| h.amplitude * (h.k as f32 * theta + h.phase).sin())
            .sum::<f32>()
    }

    /// Builds a closed SVG path through the points with Catmull-Rom splines.
    ///
    /// Each segment P1→P2 becomes a cubic Bezier with control points
    /// P1 + (P2 − P0) / 6 and P2 − (P3 − P1) / 6, which is the uniform Catmull-Rom spline.
    fn closed_catmull_rom_path(points: &[(f32, f32)]) -> String {
        let n = points.len();
        let fmt = |v: f32| ((v * 10.0).round() / 10.0).to_string();
        let mut d = format!("M {} {}", fmt(points[0].0), fmt(points[0].1));
        for i in 0..n {
            let p0 = points[(i + n - 1) % n];
            let p1 = points[i];
            let p2 = points[(i + 1) % n];
            let p3 = points[(i + 2) % n];
            let c1 = (p1.0 + (p2.0 - p0.0) / 6.0, p1.1 + (p2.1 - p0.1) / 6.0);
            let c2 = (p2.0 - (p3.0 - p1.0) / 6.0, p2.1 - (p3.1 - p1.1) / 6.0);
            d.push_str(&format!(
                " C {} {} {} {} {} {}",
                fmt(c1.0),
                fmt(c1.1),
                fmt(c2.0),
                fmt(c2.1),
                fmt(p2.0),
                fmt(p2.1)
            ));
        }
        d.push_str(" Z");
        d
    }

    /// Adds the default background: a gray circle.
    pub fn default_background(&mut self) -> &mut Self {
        let half = (self.width / 2) as f32;
        self.add_circle(HatchPatternType::Fill, Color::rgb(128, 128, 128), half, half, half)
    }

    /// Adds the default icon: a gray circle and a foreground curve from the signature.
    pub fn default_icon(&mut self, signature: &str, style: ForegroundStyle) -> &mut Self {
        self.default_background().add_foreground(signature, style)
    }

    /// Returns the SVG string. No XML declaration.
    pub fn to_svg(&self) -> String {
        let defs = if self.pattern_defs.is_empty() {
            String::new()
        } else {
            format!("<defs>{}</defs>", self.pattern_defs.concat())
        };
        format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="{vx} {vy} {w} {h}">{defs}<g>{bg}</g><g>{fg}</g></svg>"#,
            w = self.width,
            h = self.height,
            vx = -self.width / 2,
            vy = -self.height / 2,
            bg = self.bg_group.concat(),
            fg = self.fg_group.concat(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_blocks_counts_ascii_as_one() {
        let blocks = IconBuilder::split_blocks("apple");
        assert_eq!(blocks, ["app", "le", "", ""].map(String::from));
    }

    #[test]
    fn split_blocks_counts_kanji_as_two() {
        let blocks = IconBuilder::split_blocks("日本語");
        assert_eq!(blocks, ["日本", "語", "", ""].map(String::from));
    }

    #[test]
    fn split_blocks_puts_rest_in_last_block() {
        let blocks = IconBuilder::split_blocks("abcdefghijklmnop");
        assert_eq!(blocks, ["abc", "def", "ghi", "jklmnop"].map(String::from));
    }

    #[test]
    fn hiragana_is_not_ideogram() {
        assert!(!IconBuilder::is_ideogram('あ'));
        assert!(!IconBuilder::is_ideogram('ア'));
        assert!(IconBuilder::is_ideogram('漢'));
        assert!(!IconBuilder::is_ideogram('a'));
    }

    #[test]
    fn hash_u32_reads_little_endian_prefix() {
        // SHA-1("") = da39a3ee...
        assert_eq!(IconBuilder::hash_u32(""), 0xEEA3_39DA);
    }

    #[test]
    fn same_tag_gives_same_svg() {
        let a = crate::generate_ptag("apple", 100, 100);
        let b = crate::generate_ptag("apple", 100, 100);
        assert_eq!(a, b);
        assert!(a.starts_with(r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100" viewBox="-50 -50 100 100">"#));
    }

    #[test]
    fn long_tag_adds_sub_circles_and_pattern() {
        let mut builder = IconBuilder::new(100, 100);
        builder.generate_bg_from_text("abcdefghijkl");
        // Two sub circles plus the main shape make three background elements.
        assert_eq!(builder.bg_group.len(), 3);
        let svg = builder.to_svg();
        // Block 2 exists, so a hatch pattern may be used. If it is not Fill, defs must appear.
        let uses_pattern = svg.contains("url(#pat");
        assert_eq!(uses_pattern, svg.contains("<defs>"));
    }

    #[test]
    fn empty_tag_gives_default_background_without_curve() {
        let svg = crate::generate_ptag("", 100, 100);
        assert!(svg.contains(r##"<circle cx="50" cy="50" r="50" fill="#808080"/>"##));
        assert!(!svg.contains("<path "));
    }

    #[test]
    fn default_icon_has_gray_circle_and_curve() {
        let mut builder = IconBuilder::new(100, 100);
        builder.default_icon("sign", ForegroundStyle::Crossing);
        let svg = builder.to_svg();
        assert!(svg.contains(r##"fill="#808080""##));
        assert!(svg.contains("<path d=\"M "));
    }

    #[test]
    fn foreground_adds_a_path_even_for_tiny_icons() {
        let mut builder = IconBuilder::new(2, 2);
        builder.add_foreground("sign", ForegroundStyle::Crossing);
        assert_eq!(builder.fg_group.len(), 1);
    }

    #[test]
    fn same_signature_gives_same_curve() {
        let mut a = IconBuilder::new(100, 100);
        let mut b = IconBuilder::new(100, 100);
        a.add_foreground("Hidekazu", ForegroundStyle::Crossing);
        b.add_foreground("Hidekazu", ForegroundStyle::Crossing);
        assert_eq!(a.fg_group, b.fg_group);
    }

    #[test]
    fn different_signatures_give_different_curves() {
        let mut a = IconBuilder::new(100, 100);
        let mut b = IconBuilder::new(100, 100);
        a.add_foreground("alice", ForegroundStyle::Crossing);
        b.add_foreground("bob", ForegroundStyle::Crossing);
        assert_ne!(a.fg_group, b.fg_group);
    }

    #[test]
    fn foreground_path_has_one_segment_per_sample() {
        let mut builder = IconBuilder::new(100, 100);
        builder.add_foreground("sign", ForegroundStyle::Crossing);
        let path = &builder.fg_group[0];
        assert_eq!(path.matches(" C ").count(), FOREGROUND_SAMPLES);
        assert!(path.contains(" Z\""));
    }

    #[test]
    fn plain_limacon_has_single_loop() {
        let looped = [Harmonic { k: 1, amplitude: 1.5, phase: 0.0 }];
        assert!(IconBuilder::has_single_loop(&looped));
        // Amplitude below 1 never reaches the origin, so no loop.
        let convex = [Harmonic { k: 1, amplitude: 0.5, phase: 0.0 }];
        assert!(!IconBuilder::has_single_loop(&convex));
        // A strong k = 2 term crosses at antipodal angles, not once.
        let antipodal = [Harmonic { k: 2, amplitude: 1.5, phase: 0.0 }];
        assert!(!IconBuilder::has_single_loop(&antipodal));
    }

    #[test]
    fn drawn_harmonics_always_have_single_loop() {
        for seed in 0..200u64 {
            let mut rng = Pcg32::seed_from_u64(seed);
            let harmonics = IconBuilder::draw_single_loop_harmonics(&mut rng);
            assert!(IconBuilder::has_single_loop(&harmonics), "seed {seed}");
        }
    }

    #[test]
    fn no_crossing_harmonics_keep_radius_positive() {
        for seed in 0..200u64 {
            let mut rng = Pcg32::seed_from_u64(seed);
            let harmonics = IconBuilder::draw_no_crossing_harmonics(&mut rng);
            for i in 0..360 {
                let theta = i as f32 * (PI as f32) / 180.0;
                assert!(IconBuilder::radial_profile(theta, &harmonics) > 0.0, "seed {seed}");
            }
            assert!(!IconBuilder::has_single_loop(&harmonics), "seed {seed}");
        }
    }

    #[test]
    fn foreground_fits_inside_icon() {
        for (signature, style) in [
            ("a", ForegroundStyle::Crossing),
            ("Hidekazu", ForegroundStyle::Crossing),
            ("sosuisen", ForegroundStyle::NoCrossing),
            ("署名", ForegroundStyle::NoCrossing),
        ] {
            let mut builder = IconBuilder::new(100, 100);
            builder.add_foreground(signature, style);
            let path = &builder.fg_group[0];
            let d = path.split("d=\"").nth(1).unwrap().split('"').next().unwrap();
            let coords: Vec<f32> = d
                .split(' ')
                .filter_map(|t| t.parse::<f32>().ok())
                .collect();
            assert!(!coords.is_empty());
            // Every anchor and control point stays within the half size of 50.
            assert!(coords.iter().all(|v| v.abs() <= 50.5), "{signature}: {d}");
        }
    }

    #[test]
    fn some_signature_produces_a_dent() {
        // A dent exists when the radius dips below both neighbours on the sampled outline.
        let harmonics = [Harmonic { k: 5, amplitude: 0.2, phase: 0.0 }];
        let r: Vec<f32> = (0..72)
            .map(|i| IconBuilder::radial_profile(i as f32 * (PI as f32) / 36.0, &harmonics))
            .collect();
        let dents = (0..72)
            .filter(|&i| r[i] < r[(i + 71) % 72] && r[i] < r[(i + 1) % 72])
            .count();
        assert_eq!(dents, 5);
    }

    #[test]
    fn hash_u64_reads_little_endian_prefix() {
        // SHA-1("") = da39a3ee5e6b4b0d...
        assert_eq!(IconBuilder::hash_u64(""), 0x0D4B_6B5E_EEA3_39DA);
    }
}
