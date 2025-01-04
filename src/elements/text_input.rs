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
    AvailableSpace, Layout, NodeId, Size, Style,
};
use vello::kurbo::{Affine, Line, Stroke};
use winit::{
    event::{KeyEvent, MouseButton},
    keyboard::{Key, NamedKey},
    window::CursorIcon,
};

use clipboard::ClipboardProvider;
#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;
#[cfg(target_arch = "wasm32")]
use web_time::Instant;

pub struct TextInputProps {
    pub size: f64,
    pub font: &'static FontResource<'static>,
    pub placeholder: Option<String>,

    pub getter: Box<dyn Fn(&GetterContext) -> String>,
    pub setter: Box<dyn Fn(&mut EventContext, &str)>,
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
        let mut leftover = 0;

        for char in self.measurements.iter().skip(self.view) {
            total += char;

            if total > max + 0.1 {
                end = end.saturating_sub(1);
                break;
            }

            end += 1;
        }

        for char in self.measurements.iter().take(self.view).rev() {
            total += char;

            if total > max + 0.1 {
                break;
            }

            leftover += 1;
        }

        let cursor = self.selection(); // Selection end or cursor
        if cursor < self.view {
            self.view = cursor;
        } else if cursor > end + self.view {
            self.view = cursor - end;
        } else {
            // Don't waste space
            self.view = self.view.saturating_sub(leftover);
        }
    }

    /// Recompute the text measurements and mark the element as dirty
    fn reset(&mut self, ctx: &mut EventContext) {
        self.measurements =
            Text::measure_chars(self.text.chars(), self.props.size, self.props.font);

        let node = self.node_id;
        ctx.state.modify_tree(move |tree, ctx| {
            tree.mark_dirty(node).unwrap();
            ctx.state.request_redraw();
        });
    }

    /// Select the entire text
    fn select_all(&mut self) {
        let len = self.text.chars().count();

        self.selection = -(len as isize);
        self.cursor = len;

        self.view = 0;
        self.update_view();
    }

    /// Get the byte index of the character at the given index
    fn idx(&self, char: usize) -> usize {
        self.text
            .char_indices()
            .nth(char)
            .map(|(idx, _)| idx)
            .unwrap_or(self.text.len())
    }

    /// Get the byte index of the character at the cursor position
    fn cursor_idx(&self) -> usize {
        self.idx(self.cursor)
    }

    /// Get the selected text
    fn selection_to_string(&self) -> String {
        let (start, end) = self.selection_range();
        self.text.chars().skip(start).take(end - start).collect()
    }

    /// Replace the selected text with something else
    fn replace_text(&mut self, text: &str) {
        let (start, end) = self.selection_range();

        if self.selection != 0 {
            self.cursor = start + text.chars().count();
            self.selection = 0;

            let start = self.idx(start);
            let end = self.idx(end);

            self.text.replace_range(start..end, text);
        } else {
            if self.cursor < self.text.chars().count() {
                self.text.insert_str(self.cursor_idx(), text);
            } else {
                self.text.push_str(text);
            }

            self.move_cursor(text.chars().count() as isize);
        }
    }

    /// Move the cursor by a given offset
    fn move_cursor(&mut self, val: isize) {
        self.cursor = if val.is_negative() {
            self.cursor.saturating_sub(val.unsigned_abs())
        } else {
            self.cursor
                .saturating_add(val as usize)
                .min(self.text.chars().count())
        };
    }

    /// Returns the category of the given character - different category marks a word boundary
    fn char_category(c: char) -> u8 {
        match c {
            c if c.is_whitespace() => 0, // Whitespace
            c if c.is_alphanumeric() => 1, // Alphanumeric characters (letters, digits)
            '.' | ',' | ';' | ':' | '!' | '?' | '"' | '\'' => 2, // Standard punctuation
            '+' | '=' | '*' | '/' | '&' | '|' | '^' | '%' | '$' | '#' | '@' | '~' => 3, // Symbols and operators
            '[' | ']' | '(' | ')' | '{' | '}' | '<' | '>' => 4, // Brackets and parentheses
            '_' | '-' => 5, // Underscore and hyphen
            _ => 6, // Other characters (e.g., emojis, special symbols)
        }
    }

    /// Finds the closest word boundary in the given direction
    fn word_boundary_offset(&self, pos: usize, left: bool) -> usize {
        let boundary = |a: u8, b: u8| a != b || a == 0 || b == 0;

        if left {
            // Iterate backwards to find the closest boundary
            let mut last_category = None;
            let iter = self
                .text
                .chars()
                .enumerate()
                .take(pos)
                .collect::<Vec<_>>()
                .into_iter()
                .rev(); // Reverse iterator
            let mut last_boundary = 0;

            for (i, c) in iter {
                let current_category = Self::char_category(c);

                // Skip consecutive characters of the same category (including spaces)
                if let Some(last_cat) = last_category {
                    if current_category != last_cat && boundary(last_cat, current_category) {
                        last_boundary = i + 1;
                        break;
                    }
                }

                last_category = Some(current_category);
            }

            // Ensure that if we hit whitespace and then a word, we skip over the spaces entirely
            if Some(0) == last_category {
                // If the last character was whitespace, keep searching
                self.word_boundary_offset(last_boundary, left)
            } else {
                last_boundary
            }
        } else {
            // Iterate forwards to find the closest boundary, skipping current word and spaces
            let mut last_category = None;
            let iter = self.text.chars().enumerate().skip(pos);
            let mut first_boundary = self.text.len();

            let mut skip_spaces = false; // Flag to skip over spaces after current word

            for (i, c) in iter {
                let current_category = Self::char_category(c);

                // If we encounter a space and we need to skip it, just continue
                if current_category == 0 && last_category != Some(0) && last_category.is_some() {
                    skip_spaces = true;
                    continue;
                }

                // Skip consecutive characters of the same category (including spaces)
                if let Some(last_cat) = last_category {
                    if skip_spaces
                        || (current_category != last_cat && boundary(last_cat, current_category))
                    {
                        first_boundary = i;
                        break;
                    }
                }

                last_category = Some(current_category);
            }

            first_boundary
        }
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

        self.update_view();

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

        Text::new(text, self.layout, font_size, self.props.font, color, true).draw(ctx.c);

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

        let get_offset = |left, pos: usize| {
            if ctx.state.word_move_modifier() {
                self.word_boundary_offset(pos, left) as isize - pos as isize
            } else if left {
                -1
            } else {
                1
            }
        };

        match event.logical_key {
            Key::Named(NamedKey::Backspace) | Key::Named(NamedKey::Delete)
                if self.selection != 0 =>
            {
                let size = end - start;

                let start_idx = self.idx(start);
                let end_idx = self.idx(end);

                self.text.replace_range(start_idx..end_idx, "");

                self.cursor = start;
                self.view = self.view.saturating_sub(size);

                self.selection = 0;
            }

            Key::Named(NamedKey::Backspace) | Key::Named(NamedKey::Delete) => {
                let left = event.logical_key == Key::Named(NamedKey::Backspace);
                let offset = get_offset(left, self.cursor);

                self.selection = offset;
                self.replace_text("");
                self.view = self.view.saturating_sub(offset.unsigned_abs());
            }

            Key::Named(NamedKey::ArrowLeft) | Key::Named(NamedKey::ArrowRight) => {
                let left = event.logical_key == Key::Named(NamedKey::ArrowLeft);

                if ctx.state.modifiers.shift_key() {
                    let cur = self.cursor as isize;
                    let jump = get_offset(left, (cur + self.selection) as usize);

                    self.selection = (self.selection + jump)
                        .clamp(-cur, self.text.chars().count() as isize - cur);
                } else if self.selection != 0 {
                    self.cursor = if left { start } else { end };
                    self.selection = 0;
                } else {
                    self.move_cursor(get_offset(left, self.cursor));
                }

                self.cursor_opacity.reset(1.);
                self.update_view();

                ctx.state.request_redraw();

                return true;
            }

            Key::Named(NamedKey::Home)
            | Key::Named(NamedKey::PageUp)
            | Key::Named(NamedKey::End)
            | Key::Named(NamedKey::PageDown) => {
                let left = event.logical_key == Key::Named(NamedKey::Home)
                    || event.logical_key == Key::Named(NamedKey::PageUp);
                let cur = if left { 0 } else { self.text.chars().count() };

                if ctx.state.modifiers.shift_key() {
                    self.selection = cur as isize - self.cursor as isize;
                } else {
                    self.selection = 0;
                    self.cursor = cur;
                }

                self.view = cur;
                self.update_view();

                ctx.state.request_redraw();
                return true;
            }

            Key::Character(ch) if ctx.state.main_modifier() => {
                if ch == "a" {
                    self.select_all();
                    return true;
                }

                if ch == "c" || ch == "x" {
                    ctx.state
                        .clipboard
                        .set_contents(if self.selection != 0 {
                            self.selection_to_string()
                        } else {
                            self.text.clone()
                        })
                        .unwrap();

                    if ch == "c" {
                        return true;
                    }
                }

                if ch == "v" || ch == "x" {
                    let text = if ch == "v" {
                        ctx.state.clipboard.get_contents().unwrap_or_default()
                    } else {
                        String::new()
                    };

                    self.replace_text(&text);

                    self.reset(ctx);
                    self.update_view();

                    (self.props.setter)(ctx, &self.text);
                    ctx.state.request_redraw();

                    return true;
                }
            }

            _ => {
                if let Some(text) = event.text {
                    // Remove control characters
                    let text = text.chars().filter(|c| !c.is_control()).collect::<String>();
                    if text.is_empty() {
                        return false;
                    }

                    self.replace_text(&text);
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
        _: Size<Option<f32>>,
        available_space: Size<AvailableSpace>,
        _: &Style,
        _: &mut EventContext,
    ) -> Size<f32> {
        let mut text = &self.text;
        if text.is_empty() && self.props.placeholder.is_some() {
            text = self.props.placeholder.as_ref().unwrap();
        }

        let size = Text::measure(text, self.props.size, self.props.font);
        let mut width = size.x as f32;
        if let AvailableSpace::Definite(max) = available_space.width {
            width = width.min(max);
        };

        if matches!(available_space.width, AvailableSpace::MinContent) {
            width = 0.;
        }

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
                max_size: Size {
                    width: auto(),
                    height: length((props.size * 1.2) as f32),
                },
                flex_grow: 1.,
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
