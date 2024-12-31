use super::{
    node::ElementWithProps,
    primitives::{
        icon::{Icon, Symbol},
        simple_box::SimpleBox,
        traits::Draw,
    },
    tooltip::{TooltipPosition, TooltipState},
    Node,
};
use crate::{
    animations::{
        animated_property::AnimatedProperty,
        standard_animation::{Easing, StandardAnimation},
    },
    app::{
        context::{EventContext, GetterContext, RenderContext},
        ctx, EventTarget, Tree,
    },
    geometry::{Point, Rect},
};
use derive_macros::AnimatedElement;
use std::time::Duration;
use taffy::{
    prelude::{auto, length},
    Layout, NodeId, Style,
};
use winit::window::CursorIcon;

type Items = Vec<(Symbol, &'static str)>; // (Icon, Tooltip)
type Getter = Box<dyn Fn(&GetterContext) -> usize>;
type Setter = Box<dyn Fn(&mut EventContext, usize)>;

pub struct SegmentedControlProps {
    pub items: Items,
    pub getter: Getter,
    pub setter: Setter,
}

#[derive(AnimatedElement)]
pub struct SegmentedControl {
    layout: Layout,

    props: SegmentedControlProps,

    prev_index: Option<usize>,

    hover_opacity: AnimatedProperty<StandardAnimation<f32>>,
    hover_index: AnimatedProperty<StandardAnimation<f64>>,
    index: AnimatedProperty<StandardAnimation<f64>>,
}

impl SegmentedControl {
    fn cursor_to_index(&self, cursor: Point) -> usize {
        let rect: Rect = self.layout.into();
        let relative = cursor - rect.origin;

        let size = rect.size.x / self.props.items.len() as f64;
        let index = (relative.x / size).floor() as usize;

        index.clamp(0, self.props.items.len() - 1)
    }

    fn rect_at_index(&self, index: f64) -> Rect {
        let rect: Rect = self.layout.into();

        Rect::new(rect.origin + (1., 1.) + (index * 24., 0.), (22., 22.))
    }
}

impl EventTarget for SegmentedControl {
    fn update(&mut self, ctx: &mut EventContext) {
        let index = self.props.getter.as_ref()(ctx!(ctx => GetterContext));
        self.index.set(index as f64);

        if self.animate() {
            ctx.state.request_redraw();
        }
    }

    fn render(&self, ctx: &mut RenderContext) {
        let rect: Rect = self.layout.into();

        // Background
        SimpleBox::new(rect, 5., ctx.c.colors().border).draw(ctx.c);

        // Hover item
        let hover = ctx.c.colors().hover.multiply_alpha(*self.hover_opacity);
        SimpleBox::new(self.rect_at_index(*self.hover_index), 5., hover).draw(ctx.c);

        // Selected item
        SimpleBox::new(
            self.rect_at_index(*self.index),
            4.,
            ctx.c.colors().floating_background,
        )
        .draw(ctx.c);

        // Icons
        let mut icon = Rect::new(rect.origin + (4., 4.), (16., 16.));

        for (symbol, _) in &self.props.items {
            Icon::new(*symbol, icon, 16., ctx.c.colors().workspace_text).draw(ctx.c);
            icon.origin.x += 24.; // 16 = icon size, 8 = gap
        }
    }

    fn cursor(&self, _: &GetterContext) -> Option<CursorIcon> {
        Some(CursorIcon::Pointer)
    }

    fn tooltip(&self, ctx: &GetterContext) -> Option<TooltipState> {
        let index = self.cursor_to_index(ctx.state.cursor);
        let (_, tooltip) = &self.props.items[index];

        Some(TooltipState {
            text: tooltip.to_string(),
            anchor: self.rect_at_index(index as f64),
            position: TooltipPosition::Bottom,
        })
    }

    fn on_click(&mut self, ctx: &mut EventContext) -> bool {
        let index = self.cursor_to_index(ctx.state.cursor);

        self.props.setter.as_ref()(ctx, index);
        self.index.set(index as f64);

        ctx.state.tooltip_state = None;
        ctx.state.request_redraw();

        true
    }

    fn on_mouseenter(&mut self, ctx: &mut EventContext) -> bool {
        let index = self.cursor_to_index(ctx.state.cursor);
        self.hover_index.reset(index as f64);

        self.hover_opacity.set(0.1);
        ctx.state.request_redraw();

        true
    }

    fn on_mousemove(&mut self, ctx: &mut EventContext, cursor: Point) -> bool {
        let index = self.cursor_to_index(cursor);

        if Some(index) != self.prev_index {
            self.prev_index = Some(index);
            self.hover_index.set(index as f64);
            ctx.state.request_tooltip_update();
            ctx.state.request_redraw();
        }

        true
    }

    fn on_mouseleave(&mut self, ctx: &mut EventContext) -> bool {
        self.hover_opacity.set(0.);
        self.prev_index = None;

        ctx.state.request_tooltip_update();
        ctx.state.request_redraw();

        true
    }
}

impl Node for SegmentedControl {
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
        let len = self.props.items.len();
        let width = len * 16 + (len - 1) * 8 + 8; // 16 = icon size, 8 = gap, + 8 = padding

        taffy::Size {
            width: length(width as f32),
            height: length(24.),
        }
    }
}

impl ElementWithProps for SegmentedControl {
    type Props = SegmentedControlProps;

    fn setup(tree: &mut Tree, ctx: &mut EventContext, props: Self::Props) -> NodeId {
        macro_rules! animation {
            () => {
                StandardAnimation::new(Duration::from_millis(100), Easing::EaseInOut)
            };
        }

        tree.add_element(
            ctx,
            Style {
                size: taffy::Size {
                    width: auto(),
                    height: length(24.),
                },
                ..<_>::default()
            },
            None,
            |_, _| Self {
                layout: Default::default(),

                props,

                prev_index: None,

                hover_opacity: AnimatedProperty::new(animation!()),
                hover_index: AnimatedProperty::new(animation!()),
                index: AnimatedProperty::new(animation!()),
            },
        )
    }
}
