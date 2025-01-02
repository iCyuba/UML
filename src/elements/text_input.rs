use super::{
    node::ElementWithProps,
    primitives::{simple_box::SimpleBox, text::Text, traits::Draw},
    Node,
};
use crate::{
    animations::{
        animated_property::AnimatedProperty,
        standard_animation::{Easing, StandardAnimation},
    },
    app::{
        context::{EventContext, GetterContext, RenderContext},
        ctx, EventTarget, State, Tree,
    },
    geometry::{Point, Rect},
    presentation::FontResource,
};
use derive_macros::AnimatedElement;
use std::time::Duration;
use taffy::{
    prelude::{auto, length},
    Layout, NodeId, Size, Style,
};
use vello::kurbo::{Affine, Line, Stroke};
use winit::{
    event::{KeyEvent, MouseButton},
    keyboard::{Key, NamedKey},
    window::CursorIcon,
};

#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;
#[cfg(target_arch = "wasm32")]
use web_time::Instant;

pub struct TextInputProps {
    pub size: f64,
    pub font: &'static FontResource<'static>,
    pub placeholder: Option<String>,

    pub getter: fn(&GetterContext) -> String,
    pub setter: fn(&mut EventContext, &str),
}

#[derive(AnimatedElement)]
pub struct TextInput {
    layout: Layout,
    node_id: NodeId,

    pub props: TextInputProps,

    // Cache the text so the element can be marked as "dirty" when it changes
    text: String,
    measurements: Vec<f32>,
    cursor: usize,
    selection: isize,
    view: usize, // The index of the first visible character
    last_click: Instant,

    cursor_opacity: AnimatedProperty<StandardAnimation<f32>>,
}

impl TextInput {
    #[inline]
    fn focused(&self, state: &State) -> bool {
        state.focused == Some(self.node_id)
    }

    #[inline]
    fn selecting(&self, state: &State) -> bool {
        state.capturing == Some(self.node_id)
    }

    /// Get the other end of the selection
    #[inline]
    fn selection(&self) -> usize {
        if self.selection < 0 {
            self.cursor.wrapping_sub(self.selection.unsigned_abs())
        } else {
            self.cursor.saturating_add(self.selection as usize)
        }
    }

    /// Returns the selection range as a tuple of (start, end)
    fn selection_range(&self) -> (usize, usize) {
        let end = self.selection();

        if self.selection < 0 {
            (end, self.cursor)
        } else {
            (self.cursor, end)
        }
    }

    /// Returns the index of the character at the cursor position
    fn char_at_cursor(&self, cursor: Point) -> usize {
        let cursor = cursor - self.layout.location;

        let mut total = cursor.x as f32;
        let mut index = self.view;

        for char in self.measurements.iter().skip(index) {
            if total < char / 2. {
                break;
            }

            index += 1;
            total -= char;
        }

        index
    }

    /// Update the view based on the cursor position
    fn update_view(&mut self) {
        let max = self.layout.size.width;

        let mut total = 0.;
        let mut end: usize = 0;

        for char in self.measurements.iter().skip(self.view) {
            total += char;

            if total > max {
                end = end.saturating_sub(1);
                break;
            }

            end += 1;
        }

        let cursor = self.selection(); // Selection end or cursor
        if cursor < self.view {
            self.view = cursor;
        } else if cursor > end + self.view {
            self.view = cursor - end;
        }
    }

    /// Recompute the text measurements and mark the element as dirty
    fn reset(&mut self, ctx: &mut EventContext) {
        self.measurements =
            Text::measure_chars(self.text.chars(), self.props.size, self.props.font);

        let node = self.node_id;
        ctx.state.modify_tree(move |tree| {
            tree.mark_dirty(node).unwrap();
        });
    }

    /// Select the entire text
    fn select_all(&mut self) {
        let len = self.text.len();

        self.selection = -(len as isize);
        self.cursor = len;

        self.view = 0;
        self.update_view();
    }
}

impl EventTarget for TextInput {
    fn update(&mut self, ctx: &mut EventContext) {
        let text = (self.props.getter)(ctx!(ctx => GetterContext));

        if text != self.text {
            self.text = text;
            self.reset(ctx);
        }

        // Loop
        if *self.cursor_opacity == 0. {
            self.cursor_opacity.set(1.);
        } else if *self.cursor_opacity == 1. {
            self.cursor_opacity.set(0.);
        }

        self.animate();

        if self.focused(ctx.state) {
            ctx.state.request_redraw();
        }
    }

    fn render(&self, ctx: &mut RenderContext) {
        let scale = ctx.c.scale();

        let font_size = self.props.size;

        let rect = Rect::from(self.layout);
        let mut color = ctx.c.colors().text;
        let accent = ctx.c.colors().accent;

        let offset = if self.focused(ctx.state) {
            self.view
        } else {
            0
        };

        let text = if self.text.is_empty() && self.props.placeholder.is_some() {
            color = ctx.c.colors().text_secondary;

            self.props.placeholder.as_ref().unwrap()
        } else {
            &self.text.chars().skip(offset).collect::<String>()
        };

        Text::new(text, self.layout, font_size, self.props.font, color).draw(ctx.c);

        if !self.focused(ctx.state) {
            return;
        }

        // Selection
        if self.selection != 0 {
            let (start, end) = self.selection_range();

            let origin = self
                .measurements
                .iter()
                .take(start)
                .skip(offset)
                .sum::<f32>() as f64;

            let size = self
                .measurements
                .iter()
                .skip(start)
                .take(end - start)
                .sum::<f32>() as f64;

            let size = size.min(rect.size.x - origin);

            let rect = Rect::new(
                rect.origin + (origin, font_size * 0.1),
                (size, font_size * 1.2),
            );

            SimpleBox::new(rect, 3., accent.multiply_alpha(0.2)).draw(ctx.c);
        }
        // Cursor
        else {
            let offset = self
                .measurements
                .iter()
                .skip(offset)
                .take(self.cursor - offset)
                .sum::<f32>() as f64;

            ctx.c.scene().stroke(
                &Stroke::new(1. * scale),
                Affine::scale(scale).then_translate((rect.origin * scale).into()),
                accent.multiply_alpha(*self.cursor_opacity),
                None,
                &Line::new((offset, font_size * 0.1), (offset, rect.size.y)),
            );
        }
    }

    fn cursor(&self, _: &GetterContext) -> Option<CursorIcon> {
        Some(CursorIcon::Text)
    }

    fn on_click(&mut self, ctx: &mut EventContext) -> bool {
        let now = Instant::now();

        if now - self.last_click < Duration::from_millis(200) {
            self.select_all();
            ctx.state.request_redraw();
        }

        self.last_click = now;

        true
    }

    fn on_keydown(&mut self, ctx: &mut EventContext, event: KeyEvent) -> bool {
        let (start, end) = self.selection_range();

        match event.logical_key {
            Key::Named(NamedKey::Backspace) | Key::Named(NamedKey::Delete)
                if self.selection != 0 =>
            {
                let size = end - start;

                self.text.replace_range(start..end, "");

                self.cursor = start;
                self.view = self.view.saturating_sub(size);

                self.selection = 0;
            }

            Key::Named(NamedKey::Backspace) => {
                if self.cursor > 0 {
                    self.text.remove(self.cursor - 1);
                    self.cursor -= 1;
                    self.view = self.view.saturating_sub(1);
                }
            }

            Key::Named(NamedKey::Delete) => {
                if self.cursor < self.text.len() {
                    self.text.remove(self.cursor);
                }
            }

            Key::Named(NamedKey::ArrowLeft) | Key::Named(NamedKey::ArrowRight) => {
                let left = event.logical_key == Key::Named(NamedKey::ArrowLeft);

                if ctx.state.modifiers.shift_key() {
                    let jump = if left { -1 } else { 1 };
                    self.selection += jump;

                    // This will overflow when below 0
                    if self.selection() > self.text.len() {
                        self.selection -= jump;
                    }
                } else if self.selection != 0 {
                    self.cursor = if left { start } else { end };
                    self.selection = 0;
                } else {
                    self.cursor = if left {
                        self.cursor.saturating_sub(1)
                    } else {
                        self.cursor.saturating_add(1).min(self.text.len())
                    };
                }

                self.cursor_opacity.reset(1.);
                self.update_view();

                ctx.state.request_redraw();

                return true;
            }

            Key::Character(ch) if ctx.state.main_modifier() && ch == "a" => {
                self.select_all();

                ctx.state.request_redraw();
                return true;
            }

            _ => {
                if let Some(text) = event.text {
                    // Remove control characters
                    let text = text.chars().filter(|c| !c.is_control()).collect::<String>();
                    if text.is_empty() {
                        return false;
                    }

                    if self.selection != 0 {
                        self.text.replace_range(start..end, &text);
                        self.cursor = start + text.len();
                        self.selection = 0;
                    } else {
                        if self.cursor < self.text.len() {
                            self.text.insert_str(self.cursor, &text);
                        } else {
                            self.text.push_str(&text);
                        }

                        self.cursor += text.len();
                    }
                }
            }
        }

        self.reset(ctx);
        self.update_view();

        (self.props.setter)(ctx, &self.text);
        ctx.state.request_redraw();

        true
    }

    fn on_mousedown(&mut self, ctx: &mut EventContext, button: MouseButton) -> bool {
        if button != MouseButton::Left {
            return false;
        }

        ctx.state.capturing = Some(self.node_id);

        // Reset selection
        self.selection = 0;

        // If unfocused, reset the view and focus
        if !self.focused(ctx.state) {
            self.view = 0;
            self.cursor = 0;
            ctx.state.focused = Some(self.node_id);
        }

        // Cursor relative to the text element
        let pos = self.char_at_cursor(ctx.state.cursor);
        if ctx.state.modifiers.shift_key() {
            self.selection = pos as isize - self.cursor as isize;
        } else {
            self.cursor = pos;
        }

        self.cursor_opacity.reset(1.);

        ctx.state.request_redraw();

        true
    }

    fn on_mousemove(&mut self, ctx: &mut EventContext, cursor: Point) -> bool {
        if self.selecting(ctx.state) {
            let cursor = self.char_at_cursor(cursor) as isize;
            self.selection = cursor - self.cursor as isize;

            self.update_view();

            ctx.state.request_redraw();

            true
        } else {
            false
        }
    }

    fn on_mouseup(&mut self, ctx: &mut EventContext, button: MouseButton) -> bool {
        if button == MouseButton::Left {
            ctx.state.capturing = None;

            true
        } else {
            false
        }
    }
}

impl Node for TextInput {
    fn layout(&self) -> &Layout {
        &self.layout
    }

    fn layout_mut(&mut self) -> &mut Layout {
        &mut self.layout
    }

    fn measure(
        &self,
        _: taffy::Size<Option<f32>>,
        _: taffy::Size<taffy::AvailableSpace>,
        _: &Style,
        _: &mut EventContext,
    ) -> taffy::Size<f32> {
        let mut text = &self.text;
        if text.is_empty() && self.props.placeholder.is_some() {
            text = self.props.placeholder.as_ref().unwrap();
        }

        let size = Text::measure(text, self.props.size, self.props.font);
        let width = size.x as f32;

        Size {
            width,
            height: self.props.size as f32 * 1.2,
        }
    }
}

impl ElementWithProps for TextInput {
    type Props = TextInputProps;

    fn setup(tree: &mut Tree, ctx: &mut EventContext, props: TextInputProps) -> NodeId {
        tree.add_element(
            ctx,
            Style {
                size: Size {
                    width: auto(),
                    height: length((props.size * 1.2) as f32),
                },
                ..<_>::default()
            },
            None,
            |node_id, _| TextInput {
                layout: Default::default(),
                node_id,
                props,
                text: String::new(),
                measurements: vec![],
                cursor: 0,
                selection: 0,
                view: 0,
                last_click: Instant::now(),
                cursor_opacity: AnimatedProperty::new(StandardAnimation::initialized(
                    1.,
                    Duration::from_millis(400),
                    Easing::EaseInOut,
                )),
            },
        )
    }
}
