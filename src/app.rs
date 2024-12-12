use crate::renderer::WindowRenderer;
use crate::workspace::Workspace;
use winit::application::ApplicationHandler;
use winit::event::{MouseScrollDelta, WindowEvent};
use winit::event_loop::ActiveEventLoop;

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
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.renderer.window.is_none() {
            self.renderer.init(event_loop);
        }
    }

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
                match delta {
                    MouseScrollDelta::LineDelta(x, y) => {
                        self.workspace.update_position(x as f64 * 32., y as f64 * 32.);
                    }

                    MouseScrollDelta::PixelDelta(position) => {
                        self.workspace.update_position(position.x, position.y);
                    }
                }

                self.renderer.request_redraw();
            }

            WindowEvent::PinchGesture { delta, .. } => {
                self.workspace.update_zoom(delta);
                self.renderer.request_redraw();
            }

            WindowEvent::CursorMoved { position, .. } => {
                self.workspace.update_cursor(position);
            }

            _ => {}
        }
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        // Do nothing
    }
}
