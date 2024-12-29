use super::renderer::Canvas;
use crate::{app::State, data::Project};

pub struct Context<P: AsRef<Project>, S: AsRef<State>, C: AsRef<Canvas>> {
    pub project: P,
    pub state: S,

    /// Canvas
    pub c: C,
}

pub type MutContext<'a, 'b> = Context<&'a mut Project, &'a mut State, &'a mut Canvas>;
pub type EventContext<'a, 'b> = Context<&'a mut Project, &'a mut State, &'a Canvas>;
pub type RenderContext<'a, 'b> = Context<&'a Project, &'a State, &'a mut Canvas>;
pub type GetterContext<'a, 'b> = Context<&'a Project, &'a State, &'a Canvas>;

impl<'a, 'b> From<MutContext<'a, 'b>> for EventContext<'a, 'b> {
    fn from(ctx: MutContext<'a, 'b>) -> Self {
        Self {
            project: ctx.project,
            state: ctx.state,
            c: ctx.c,
        }
    }
}

impl<'a, 'b> From<MutContext<'a, 'b>> for RenderContext<'a, 'b> {
    fn from(ctx: MutContext<'a, 'b>) -> Self {
        Self {
            project: ctx.project,
            state: ctx.state,
            c: ctx.c,
        }
    }
}

impl<'a, 'b> From<MutContext<'a, 'b>> for GetterContext<'a, 'b> {
    fn from(ctx: MutContext<'a, 'b>) -> Self {
        Self {
            project: ctx.project,
            state: ctx.state,
            c: ctx.c,
        }
    }
}

macro_rules! ctx {
    // Expects a mutable reference to App
    ($app:expr) => {
        &mut $crate::app::context::MutContext {
            project: &mut $app.project,
            state: &mut $app.state,
            c: &mut $app.window.canvas(),
        }
        .into()
    };
}

pub(crate) use ctx;
