use crate::{
    app::{Renderer, State},
    data::Project,
};

pub struct Context<P, S, R> {
    pub project: P,
    pub state: S,

    /// Renderer
    pub r: R,
}

pub type MutContext<'a, 'b> = Context<&'a mut Project, &'a mut State, &'a mut Renderer<'b>>;
pub type EventContext<'a, 'b> = Context<&'a mut Project, &'a mut State, &'a Renderer<'b>>;
pub type RenderContext<'a, 'b> = Context<&'a Project, &'a State, &'a mut Renderer<'b>>;
pub type GetterContext<'a, 'b> = Context<&'a Project, &'a State, &'a Renderer<'b>>;

impl<'a, 'b> From<MutContext<'a, 'b>> for EventContext<'a, 'b> {
    fn from(ctx: MutContext<'a, 'b>) -> Self {
        Self {
            project: ctx.project,
            state: ctx.state,
            r: ctx.r,
        }
    }
}

impl<'a, 'b> From<MutContext<'a, 'b>> for RenderContext<'a, 'b> {
    fn from(ctx: MutContext<'a, 'b>) -> Self {
        Self {
            project: ctx.project,
            state: ctx.state,
            r: ctx.r,
        }
    }
}

impl<'a, 'b> From<MutContext<'a, 'b>> for GetterContext<'a, 'b> {
    fn from(ctx: MutContext<'a, 'b>) -> Self {
        Self {
            project: ctx.project,
            state: ctx.state,
            r: ctx.r,
        }
    }
}

macro_rules! ctx {
    // Expects a mutable reference to App
    ($app:expr) => {
        &mut $crate::app::context::MutContext {
            project: &mut $app.project,
            state: &mut $app.state,
            r: &mut $app.renderer,
        }
        .into()
    };
}

pub(crate) use ctx;
