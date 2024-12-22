#![allow(dead_code)]

use std::sync::{Arc, OnceLock};
use vello::peniko::{Blob, Font};
use vello::skrifa::charmap::Charmap;
use vello::skrifa::instance::{Location, Size};
use vello::skrifa::metrics::GlyphMetrics;
use vello::skrifa::{AxisCollection, FontRef, MetadataProvider};

macro_rules! font {
    ($name:ident, $file:expr, $weight:expr) => {
        pub fn $name() -> &'static FontResource<'static> {
            static BYTES: &[u8] = include_bytes!(concat!("../../assets/", $file, ".ttf"));
            static FONT: OnceLock<FontResource> = OnceLock::new();

            FONT.get_or_init(|| FontResource::from_bytes(BYTES, $weight))
        }
    };
}

// Regular
font!(inter_thin, "Inter-Thin", 100.0);
font!(inter_extra_light, "Inter-ExtraLight", 200.0);
font!(inter_light, "Inter-Light", 300.0);
font!(inter_regular, "Inter-Regular", 400.0);
font!(inter_medium, "Inter-Medium", 500.0);
font!(inter_semi_bold, "Inter-SemiBold", 600.0);
font!(inter_bold, "Inter-Bold", 700.0);
font!(inter_extra_bold, "Inter-ExtraBold", 800.0);
font!(inter_black, "Inter-Black", 900.0);

// Italic
font!(inter_thin_italic, "Inter-ThinItalic", 100.0);
font!(inter_extra_light_italic, "Inter-ExtraLightItalic", 200.0);
font!(inter_light_italic, "Inter-LightItalic", 300.0);
font!(inter_italic, "Inter-Italic", 400.0);
font!(inter_medium_italic, "Inter-MediumItalic", 500.0);
font!(inter_semi_bold_italic, "Inter-SemiBoldItalic", 600.0);
font!(inter_bold_italic, "Inter-BoldItalic", 700.0);
font!(inter_extra_bold_italic, "Inter-ExtraBoldItalic", 800.0);
font!(inter_black_italic, "Inter-BlackItalic", 900.0);

// Icons
font!(icons, "Icons", 100.0);

pub struct FontResource<'a> {
    pub table: FontRef<'a>,
    pub font: Font,
    pub char_map: Charmap<'a>,
    pub axes: AxisCollection<'a>,
    pub location: Location,
}

impl FontResource<'_> {
    pub fn from_bytes(bytes: &'static [u8], weight: f32) -> Self {
        let table = FontRef::new(bytes).unwrap();
        let font = Font::new(Blob::new(Arc::new(bytes)), 0);
        let char_map = table.charmap();
        let axes = table.axes();
        let location = axes.location([("wght", weight)]);

        Self {
            table,
            font,
            char_map,
            axes,
            location,
        }
    }

    pub fn metrics(&self, font_size: f32) -> GlyphMetrics {
        self.table
            .glyph_metrics(Size::new(font_size), &self.location)
    }
}
