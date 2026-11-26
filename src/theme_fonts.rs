//! Font setup for the PULSE Obsidian Cyan Cockpit theme.
//! Inter and JetBrains Mono are embedded from `assets/fonts` and licensed
//! under the SIL Open Font License 1.1 (see FONT_LICENSE below).

use eframe::egui;
use std::path::Path;

/// Embedded proportional typeface: Inter Regular (SIL OFL 1.1).
pub const INTER_TTF: &[u8] = include_bytes!("../assets/fonts/Inter-Regular.ttf");
/// Embedded monospace typeface: JetBrains Mono Regular (SIL OFL 1.1).
pub const JETBRAINS_MONO_TTF: &[u8] = include_bytes!("../assets/fonts/JetBrainsMono-Regular.ttf");
/// Complete license text shipped with the two embedded fonts.
pub const FONT_LICENSE: &str = include_str!("../assets/fonts/Inter-JetBrainsMono-OFL.txt");

/// Maximum bytes allowed for a single font file (16 MB).
pub const MAX_FONT_BYTES: u64 = 16_000_000;

/// TrueType files start with the big-endian magic value 0x00010000.
pub(crate) fn has_ttf_magic(bytes: &[u8]) -> bool {
    bytes.len() >= 4 && bytes[0] == 0 && bytes[1] == 1 && bytes[2] == 0 && bytes[3] == 0
}

/// Reads a font file only when its metadata reports a non-empty regular file within
/// `MAX_FONT_BYTES`. Oversized/empty/missing files are rejected before reading.
pub(crate) fn read_font_bounded(path: &Path) -> Option<Vec<u8>> {
    let meta = std::fs::metadata(path).ok()?;
    if !meta.is_file() || meta.len() == 0 || meta.len() > MAX_FONT_BYTES {
        return None;
    }
    let bytes = std::fs::read(path).ok()?;
    if bytes.len() as u64 > MAX_FONT_BYTES {
        return None;
    }
    Some(bytes)
}

/// Installs the embedded fonts as the default proportional and monospace faces.
/// System fonts are consulted only if an embedded payload ever fails validation.
pub fn setup_fonts(ctx: &egui::Context) {
    debug_assert!(FONT_LICENSE.contains("SIL OPEN FONT LICENSE"));
    let mut fonts = egui::FontDefinitions::default();

    if has_ttf_magic(INTER_TTF) {
        fonts.font_data.insert("inter".to_owned(), egui::FontData::from_owned(INTER_TTF.to_vec()));
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "inter".to_owned());
    } else if let Some(data) = read_font_bounded(Path::new(r"C:\Windows\Fonts\segoeui.ttf")) {
        fonts.font_data.insert("segoe_ui".to_owned(), egui::FontData::from_owned(data));
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "segoe_ui".to_owned());
    }

    if has_ttf_magic(JETBRAINS_MONO_TTF) {
        fonts.font_data.insert(
            "jetbrains_mono".to_owned(),
            egui::FontData::from_owned(JETBRAINS_MONO_TTF.to_vec()),
        );
        fonts
            .families
            .entry(egui::FontFamily::Monospace)
            .or_default()
            .insert(0, "jetbrains_mono".to_owned());
    } else if let Some(data) = read_font_bounded(Path::new(r"C:\Windows\Fonts\consola.ttf")) {
        fonts.font_data.insert("consolas".to_owned(), egui::FontData::from_owned(data));
        fonts
            .families
            .entry(egui::FontFamily::Monospace)
            .or_default()
            .insert(0, "consolas".to_owned());
    }

    ctx.set_fonts(fonts);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_inter_has_ttf_magic_bytes() {
        assert_eq!(&INTER_TTF[..4], &[0, 1, 0, 0]);
        assert!(has_ttf_magic(INTER_TTF));
    }

    #[test]
    fn embedded_jetbrains_mono_has_ttf_magic_bytes() {
        assert_eq!(&JETBRAINS_MONO_TTF[..4], &[0, 1, 0, 0]);
        assert!(has_ttf_magic(JETBRAINS_MONO_TTF));
    }

    #[test]
    fn bundled_notice_is_sil_open_font_license() {
        assert!(FONT_LICENSE.contains("SIL OPEN FONT LICENSE"));
        assert!(!has_ttf_magic(&[0x4F, 0x54, 0x54, 0x4F]));
    }
}
