use egui::{Color32, Id, LayerId};

/// Temporary editing details. The committed color always belongs to the host.
#[derive(Clone)]
pub(super) struct State {
    pub last: Color32,
    pub rgba: [u8; 4],
    pub hex: String,
    pub open: bool,
    pub pending_focus: bool,
    pub layer: Option<LayerId>,
    pub focused: Option<Id>,
    alpha: bool,
}

impl State {
    pub fn new(color: Color32, alpha: bool) -> Self {
        let rgba = color.to_srgba_unmultiplied();
        Self {
            last: color,
            rgba,
            hex: format_hex(rgba, alpha),
            open: false,
            pending_focus: false,
            layer: None,
            focused: None,
            alpha,
        }
    }

    pub fn sync(&mut self, color: Color32, alpha: bool) {
        if self.last != color || self.alpha != alpha {
            if self.last != color {
                self.last = color;
                self.rgba = color.to_srgba_unmultiplied();
            }
            self.hex = format_hex(self.rgba, alpha);
            self.alpha = alpha;
        }
    }

    pub fn color(&self) -> Color32 {
        Color32::from_rgba_unmultiplied(self.rgba[0], self.rgba[1], self.rgba[2], self.rgba[3])
    }

    pub fn set_preset(&mut self, color: Color32, alpha: bool) {
        let previous_alpha = self.rgba[3];
        self.rgba = color.to_srgba_unmultiplied();
        if !alpha {
            self.rgba[3] = previous_alpha;
        }
        self.hex = format_hex(self.rgba, alpha);
    }
}

pub(super) fn format_hex(rgba: [u8; 4], alpha: bool) -> String {
    let [r, g, b, a] = rgba;
    if alpha {
        format!("#{r:02X}{g:02X}{b:02X}{a:02X}")
    } else {
        format!("#{r:02X}{g:02X}{b:02X}")
    }
}

/// CSS-style sRGB hex, with straight alpha last. RGB-only mode preserves alpha.
pub(super) fn parse_hex(text: &str, alpha: bool, current_alpha: u8) -> Option<[u8; 4]> {
    let text = text.trim().strip_prefix('#').unwrap_or(text.trim());
    let bytes = text.as_bytes();
    if !bytes.iter().all(u8::is_ascii_hexdigit) {
        return None;
    }
    let has_alpha = matches!(bytes.len(), 4 | 8);
    if !matches!(bytes.len(), 3 | 4 | 6 | 8) || (has_alpha && !alpha) {
        return None;
    }
    let short = bytes.len() <= 4;
    let mut rgba = [0, 0, 0, if alpha { 255 } else { current_alpha }];
    let channels = if has_alpha { 4 } else { 3 };
    for (index, channel) in rgba.iter_mut().take(channels).enumerate() {
        let digit = |byte: u8| (byte as char).to_digit(16).expect("validated ASCII hex") as u8;
        *channel = if short {
            digit(bytes[index]) * 17
        } else {
            digit(bytes[2 * index]) * 16 + digit(bytes[2 * index + 1])
        };
    }
    Some(rgba)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_accepts_css_forms_without_slicing_unvalidated_utf8() {
        assert_eq!(parse_hex(" #f81 ", false, 64), Some([255, 136, 17, 64]));
        assert_eq!(parse_hex("F9741580", true, 1), Some([249, 116, 21, 128]));
        assert_eq!(parse_hex("#f810", true, 1), Some([255, 136, 17, 0]));
        assert_eq!(parse_hex("#112233", true, 1), Some([17, 34, 51, 255]));
        for text in ["", "#", "#12", "#12345", "##abcdef", "#gé1234", "#12345678"] {
            assert_eq!(parse_hex(text, false, 255), None, "{text}");
        }
    }
}
