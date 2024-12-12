use std::sync::{Arc, OnceLock};
use vello::peniko::{Blob, Font};
use vello::skrifa::charmap::Charmap;
use vello::skrifa::instance::{Location, Size};
use vello::skrifa::metrics::GlyphMetrics;
use vello::skrifa::{AxisCollection, FontRef, MetadataProvider};

const INTER_REGULAR_BYTES: &[u8] = include_bytes!("../assets/Inter-Regular.ttf");
const INTER_EXTRA_LIGHT_BYTES: &[u8] = include_bytes!("../assets/Inter-ExtraLight.ttf");

pub fn inter_regular() -> &'static FontResource<'static> {
    static INTER_REGULAR: OnceLock<FontResource> = OnceLock::new();

    INTER_REGULAR.get_or_init(|| {
        FontResource::from_bytes(INTER_REGULAR_BYTES, 400.0)
    })
}

pub fn inter_extra_light() -> &'static FontResource<'static> {
    static INTER_EXTRA_LIGHT: OnceLock<FontResource> = OnceLock::new();

    INTER_EXTRA_LIGHT.get_or_init(|| {
        FontResource::from_bytes(INTER_EXTRA_LIGHT_BYTES, 200.0)
    })
}

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
        self.table.glyph_metrics(Size::new(font_size), &self.location)
    }
}
