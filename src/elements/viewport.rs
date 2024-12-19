use crate::elements::workspace::Workspace;
use crate::elements::Element;
use std::iter;

pub struct Viewport {
    workspace: Workspace,
}

impl Viewport {
    pub fn new() -> Self {
        Self {
            workspace: Workspace::new(),
        }
    }
}

impl Element for Viewport {
    fn children(&self) -> Box<dyn Iterator<Item = &dyn Element> + '_> {
        Box::new(iter::once(&self.workspace as &dyn Element))
    }

    fn children_mut(&mut self) -> Box<dyn Iterator<Item = &mut dyn Element> + '_> {
        Box::new(iter::once(&mut self.workspace as &mut dyn Element))
    }
}
