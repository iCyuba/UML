use crate::app::renderer::Canvas;
use crate::elements::primitives::text::Text;
use crate::elements::primitives::traits::Draw;
use crate::geometry::Rect;
use crate::presentation::fonts;
use vello::peniko::Color;

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum Symbol {
    Cursor = b'A',
    PlusSquare = b'B',
    Workflow = b'C',
    Hand = b'D',
    Class = b'E',
    AbstractClass = b'F',
    SealedClass = b'G',
    Interface = b'H',
    Plus = b'I',
    Minus = b'J',
    Hashtag = b'K',
    Load = b'L',
    Save = b'M',
    Screenshot = b'N',
    Pen = b'O',
    One = b'P',
    Many = b'Q',
    Trash = b'R',
    OneWayAssociation = b'S',
    Aggregation = b'T',
    Composition = b'U',
    Generalization = b'V',
    Swap = b'W',
    Field = b'X',
    Method = b'Y',
    Association = b'Z',
    Export = b'a',
}

impl From<Symbol> for char {
    #[inline]
    fn from(val: Symbol) -> Self {
        val as u8 as char
    }
}

pub struct Icon {
    content: Symbol,
    color: Color,
    size: f64,
    rect: Rect,
}

impl Icon {
    pub fn new(content: Symbol, rect: Rect, size: f64, color: Color) -> Self {
        Self {
            content,
            rect,
            size,
            color,
        }
    }
}

impl Draw for Icon {
    fn draw(&self, c: &mut Canvas) {
        let mut str = [0; 1];
        Text::new(
            char::from(self.content).encode_utf8(&mut str),
            self.rect,
            self.size,
            fonts::icons(),
            self.color,
            false,
        )
        .draw(c);
    }
}
