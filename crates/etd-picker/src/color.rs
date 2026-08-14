#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RgbColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl RgbColor {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub fn to_hex(&self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }

    pub fn to_rgb_string(&self) -> String {
        format!("{}, {}, {}", self.r, self.g, self.b)
    }

    pub fn to_css_rgb(&self) -> String {
        format!("rgb({}, {}, {})", self.r, self.g, self.b)
    }

    pub fn to_hsl(&self) -> (u16, u8, u8) {
        let r = self.r as f32 / 255.0;
        let g = self.g as f32 / 255.0;
        let b = self.b as f32 / 255.0;

        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let delta = max - min;

        let mut h = 0.0;
        let mut s = 0.0;
        let l = (max + min) / 2.0;

        if delta != 0.0 {
            s = if l < 0.5 {
                delta / (max + min)
            } else {
                delta / (2.0 - max - min)
            };

            if max == r {
                h = ((g - b) / delta) + (if g < b { 6.0 } else { 0.0 });
            } else if max == g {
                h = ((b - r) / delta) + 2.0;
            } else {
                h = ((r - g) / delta) + 4.0;
            }
            h *= 60.0;
        }

        (
            h.round() as u16,
            (s * 100.0).round() as u8,
            (l * 100.0).round() as u8,
        )
    }

    pub fn to_hsv(&self) -> (u16, u8, u8) {
        let r = self.r as f32 / 255.0;
        let g = self.g as f32 / 255.0;
        let b = self.b as f32 / 255.0;

        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let delta = max - min;

        let mut h = 0.0;
        let s = if max == 0.0 { 0.0 } else { delta / max };
        let v = max;

        if delta != 0.0 {
            if max == r {
                h = ((g - b) / delta) + (if g < b { 6.0 } else { 0.0 });
            } else if max == g {
                h = ((b - r) / delta) + 2.0;
            } else {
                h = ((r - g) / delta) + 4.0;
            }
            h *= 60.0;
        }

        (
            h.round() as u16,
            (s * 100.0).round() as u8,
            (v * 100.0).round() as u8,
        )
    }

    pub fn to_cmyk(&self) -> (u8, u8, u8, u8) {
        let r = self.r as f32 / 255.0;
        let g = self.g as f32 / 255.0;
        let b = self.b as f32 / 255.0;

        let k = 1.0 - r.max(g).max(b);
        if k >= 1.0 {
            return (0, 0, 0, 100);
        }

        let c = (1.0 - r - k) / (1.0 - k);
        let m = (1.0 - g - k) / (1.0 - k);
        let y = (1.0 - b - k) / (1.0 - k);

        (
            (c * 100.0).round() as u8,
            (m * 100.0).round() as u8,
            (y * 100.0).round() as u8,
            (k * 100.0).round() as u8,
        )
    }

    pub fn from_hex(hex: &str) -> Option<Self> {
        let clean = hex.trim().trim_start_matches('#');
        if clean.len() != 6 {
            return None;
        }
        let r = u8::from_str_radix(&clean[0..2], 16).ok()?;
        let g = u8::from_str_radix(&clean[2..4], 16).ok()?;
        let b = u8::from_str_radix(&clean[4..6], 16).ok()?;
        Some(Self { r, g, b })
    }

    pub fn is_dark(&self) -> bool {
        // Perceived luminance formula
        let luminance = 0.299 * self.r as f32 + 0.587 * self.g as f32 + 0.114 * self.b as f32;
        luminance < 128.0
    }
}
