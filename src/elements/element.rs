use crate::app::EventTarget;
use taffy::Layout;

pub trait Element: EventTarget {
    // Box model
    fn layout(&self) -> &Layout;
    fn layout_mut(&mut self) -> &mut Layout;
}
