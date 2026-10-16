//! Color palette and brightness adjustment.
//!
//! The palette comes from two triad schemes picked at <https://color.adobe.com/>.
//! triad01: #EBAE9B, #9BA9EB, #C0EB9B
//! triad02: #EB9BB6, #9BDAEB, #EBE49A

/// RGB color. Alpha is not handled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Returns the color as `#RRGGBB`.
    pub fn to_hex(self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }

    /// Returns the color with its lightness multiplied by `brightness_factor`.
    pub fn change_brightness(self, brightness_factor: f64) -> Self {
        let (hue, saturation, brightness) = self.to_hsl();
        let brightness = (brightness * brightness_factor).clamp(0.0, 1.0);
        Self::from_hsl(hue, saturation, brightness)
    }

    /// Converts RGB to HSL. Returns (hue, saturation, lightness).
    fn to_hsl(self) -> (f64, f64, f64) {
        // The original divides in single precision before widening to double.
        // Keep the same precision so the results match.
        let r = (self.r as f32 / 255.0) as f64;
        let g = (self.g as f32 / 255.0) as f64;
        let b = (self.b as f32 / 255.0) as f64;

        let max = r.max(g.max(b));
        let min = r.min(g.min(b));
        let delta = max - min;

        let brightness = (max + min) / 2.0;

        let (mut hue, saturation) = if delta == 0.0 {
            (0.0, 0.0)
        } else {
            let saturation = delta / (1.0 - (2.0 * brightness - 1.0).abs());
            let hue = if max == r {
                60.0 * (((g - b) / delta) % 6.0)
            } else if max == g {
                60.0 * (((b - r) / delta) + 2.0)
            } else {
                60.0 * (((r - g) / delta) + 4.0)
            };
            (hue, saturation)
        };

        if hue < 0.0 {
            hue += 360.0;
        }
        (hue, saturation, brightness)
    }

    fn from_hsl(hue: f64, saturation: f64, brightness: f64) -> Self {
        let c = (1.0 - (2.0 * brightness - 1.0).abs()) * saturation;
        let x = c * (1.0 - ((hue / 60.0) % 2.0 - 1.0).abs());
        let m = brightness - c / 2.0;

        let (r_prime, g_prime, b_prime) = if (0.0..60.0).contains(&hue) {
            (c, x, 0.0)
        } else if (60.0..120.0).contains(&hue) {
            (x, c, 0.0)
        } else if (120.0..180.0).contains(&hue) {
            (0.0, c, x)
        } else if (180.0..240.0).contains(&hue) {
            (0.0, x, c)
        } else if (240.0..300.0).contains(&hue) {
            (x, 0.0, c)
        } else {
            (c, 0.0, x)
        };

        // The original truncates with a (byte) cast, so do not round here.
        Self::rgb(
            ((r_prime + m) * 255.0) as u8,
            ((g_prime + m) * 255.0) as u8,
            ((b_prime + m) * 255.0) as u8,
        )
    }
}

/// Named palette color.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ColorName {
    Orange,
    Blue,
    Green,
    Pink,
    Cyan,
    Yellow,
}

impl ColorName {
    pub const ALL: [ColorName; 6] = [
        ColorName::Orange,
        ColorName::Blue,
        ColorName::Green,
        ColorName::Pink,
        ColorName::Cyan,
        ColorName::Yellow,
    ];

    pub const COUNT: u32 = Self::ALL.len() as u32;

    /// Picks a color by index. Out-of-range indexes wrap around by `COUNT`.
    pub fn from_index(index: u32) -> Self {
        Self::ALL[(index % Self::COUNT) as usize]
    }

    pub fn color(self) -> Color {
        match self {
            ColorName::Orange => Color::rgb(0xEB, 0xAE, 0x9B),
            ColorName::Blue => Color::rgb(0x9B, 0xA9, 0xEB),
            ColorName::Green => Color::rgb(0xC0, 0xEB, 0x9B),
            ColorName::Pink => Color::rgb(0xEB, 0x9B, 0xB6),
            ColorName::Cyan => Color::rgb(0x9B, 0xDA, 0xEB),
            ColorName::Yellow => Color::rgb(0xEB, 0xE4, 0x9A),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_hex_is_uppercase_with_hash() {
        assert_eq!(ColorName::Orange.color().to_hex(), "#EBAE9B");
    }

    #[test]
    fn change_brightness_with_factor_one_keeps_color_close() {
        let original = ColorName::Blue.color();
        let same = original.change_brightness(1.0);
        // Truncation may shift a channel by one.
        assert!((same.r as i32 - original.r as i32).abs() <= 1);
        assert!((same.g as i32 - original.g as i32).abs() <= 1);
        assert!((same.b as i32 - original.b as i32).abs() <= 1);
    }

    #[test]
    fn change_brightness_darkens_and_lightens() {
        let original = ColorName::Green.color();
        let darker = original.change_brightness(0.9);
        let lighter = original.change_brightness(1.1);
        assert!(darker.g < original.g);
        assert!(lighter.r > original.r);
    }

    #[test]
    fn gray_has_no_hue() {
        let gray = Color::rgb(128, 128, 128);
        let (hue, saturation, _) = gray.to_hsl();
        assert_eq!(hue, 0.0);
        assert_eq!(saturation, 0.0);
    }
}
