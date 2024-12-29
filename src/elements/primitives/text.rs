#![allow(dead_code)]

use crate::app::renderer::Canvas;
use crate::elements::primitives::traits::Draw;
use crate::geometry::Rect;
use crate::presentation::FontResource;
use vello::kurbo::Affine;
use vello::peniko::{BrushRef, Fill, StyleRef};
use vello::Glyph;

pub struct Text<'a> {
    text: &'a str,

    rect: Rect,
    size: f64,

    font: &'a FontResource<'a>,
    brush: BrushRef<'a>,
}

impl<'a> Text<'a> {
    const ELLIPSIS: char = '…';

    pub fn new(
        text: &'a str,
        rect: impl Into<Rect>,
        size: f64,
        font: &'a FontResource<'a>,
        brush: impl Into<BrushRef<'a>>,
    ) -> Self {
        let rect = rect.into();
        let brush = brush.into();

        Self {
            text,
            rect,
            size,
            font,
            brush,
        }
    }

    pub fn measure(text: &str, size: f64, font: &FontResource) -> Rect {
        let metrics = font.metrics(size as f32);
        let width = text
            .chars()
            .map(|c| {
                metrics
                    .advance_width(font.char_map.map(c).unwrap_or_default())
                    .unwrap_or_default()
            })
            .sum::<f32>();

        Rect::new((0., 0.), (width as f64, size * 1.2))
    }
}

impl Draw for Text<'_> {
    fn draw(&self, c: &mut Canvas) {
        let scale = c.scale();
        let size = self.size * scale;

        let pos = self.rect.origin * scale;
        let font = self.font;
        let metrics = font.metrics(self.size as f32);

        let ellipsis = font.char_map.map(Self::ELLIPSIS).unwrap_or_default();
        let safe_width_offset = metrics.advance_width(ellipsis).unwrap_or_default() as f64;

        let last_char = self.text.len() - 1;
        let max_width = self.rect.size.x;

        c.scene()
            .draw_glyphs(&font.font)
            .font_size(size as f32)
            .brush(self.brush)
            .transform(Affine::translate((pos.x, pos.y + size)))
            .draw(
                StyleRef::Fill(Fill::NonZero),
                self.text.chars().enumerate().scan(0.0, |p_x, (i, c)| {
                    let glyph_id = font.char_map.map(c).unwrap_or_default();
                    let width = metrics.advance_width(glyph_id).unwrap_or_default() as f64;
                    let x = *p_x;

                    let overflow = x + width > max_width - safe_width_offset && i < last_char;
                    *p_x = if overflow { max_width } else { *p_x + width };

                    if x + width > max_width {
                        None
                    } else {
                        Some(Glyph {
                            id: (if overflow { ellipsis } else { glyph_id }).to_u32(),
                            x: (x * scale) as f32,
                            y: 0.,
                        })
                    }
                }),
            );
    }
}
