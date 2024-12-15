use crate::renderer::WindowRenderer;
use crate::workspace::Workspace;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::NamedKey;

#[cfg(not(target_os = "macos"))]
pub const MAIN_MODIFIER: NamedKey = NamedKey::Control;

#[cfg(target_os = "macos")]
pub const MAIN_MODIFIER: NamedKey = NamedKey::Super;

pub struct App<'s> {
    pub renderer: WindowRenderer<'s>,
    pub workspace: Workspace,
}

impl App<'_> {
    pub fn new() -> Self {
        App {
            renderer: WindowRenderer::default(),
            workspace: Workspace::new(),
        }
    }
}

impl ApplicationHandler for App<'_> {
    #[cfg(not(target_arch = "wasm32"))]
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.renderer.window.is_none() {
            self.renderer.init(event_loop);
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {}

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let Some(window) = self.renderer.window.as_ref() else {
            return;
        };

        if window.id() != window_id {
            return;
        }

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => self.renderer.resize(size),

            WindowEvent::RedrawRequested => {
                self.renderer.scene.reset();
                self.workspace.render(&mut self.renderer);

                // Render the scene
                self.renderer.render();
            }

            WindowEvent::MouseWheel { delta, .. } => {
                self.workspace.handle_scroll(delta);
                self.renderer.request_redraw();
            }

            WindowEvent::PinchGesture { delta, .. } => {
                self.workspace.update_zoom(delta);
                self.renderer.request_redraw();
            }

            WindowEvent::CursorMoved { position, .. } => {
                if self.workspace.handle_mouse_move(position) {
                    self.renderer.request_redraw();
                }
            }

            WindowEvent::ThemeChanged(theme) => {
                self.renderer.update_theme(theme);
                self.renderer.request_redraw();
            }

            WindowEvent::MouseInput { state, button, .. } => {
                let pressed = state == ElementState::Pressed;
                self.workspace.update_mouse_buttons(button, pressed);
            }

            WindowEvent::KeyboardInput { event, .. } => {
                self.workspace.update_keys(event.logical_key, event.state.is_pressed());
            }

            _ => {}
        }

        self.workspace.animate(&mut event_loop.control_flow(), &self.renderer);
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        // Do nothing
    }
}
