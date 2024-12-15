use crate::renderer::WindowRenderer;
use crate::workspace::Workspace;
use winit::application::ApplicationHandler;
#[cfg(target_arch = "wasm32")]
use winit::dpi::PhysicalPosition;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoopProxy};
use winit::keyboard::NamedKey;

#[derive(Debug)]
pub enum AppUserEvent {
    #[cfg(target_arch = "wasm32")]
    Scroll { delta: PhysicalPosition<f64>, ctrl_key: bool },
}

pub struct App<'s> {
    #[allow(dead_code)] // Unused on the desktop app rn
    event_loop: EventLoopProxy<AppUserEvent>,

    pub main_modifier: NamedKey,

    pub renderer: WindowRenderer<'s>,
    pub workspace: Workspace,
}

impl App<'_> {
    pub fn new(event_loop: EventLoopProxy<AppUserEvent>) -> Self {
        App {
            event_loop,

            #[cfg(not(target_os = "macos"))]
            main_modifier: NamedKey::Control,

            #[cfg(target_os = "macos")]
            main_modifier: NamedKey::Super,

            renderer: WindowRenderer::default(),
            workspace: Workspace::new(),
        }
    }
}

impl ApplicationHandler<AppUserEvent> for App<'_> {
    #[cfg(not(target_arch = "wasm32"))]
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.renderer.window.is_none() {
            self.renderer.init(event_loop);
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {
        use web_sys::wasm_bindgen::closure::Closure;
        use web_sys::wasm_bindgen::JsCast;

        let window = web_sys::window().unwrap();

        // Set the main modifier key
        if window.navigator().user_agent().unwrap().to_lowercase().contains("mac") {
            self.main_modifier = NamedKey::Super;
        }

        // Setup a better scroll handler
        let proxy = self.event_loop.clone();
        let main_modifier = self.main_modifier;
        let closure = Closure::wrap(Box::new(move |event: web_sys::WheelEvent| {
            proxy.send_event(AppUserEvent::Scroll {
                delta: PhysicalPosition {
                    x: -event.delta_x(),
                    y: -event.delta_y(),
                },
                ctrl_key: event.ctrl_key() ||
                    (main_modifier == NamedKey::Super && event.meta_key()),
            }).unwrap();
        }) as Box<dyn FnMut(_)>);

        window.add_event_listener_with_callback("wheel", &closure.as_ref().unchecked_ref()).unwrap();
        closure.forget();
    }

    #[cfg(target_arch = "wasm32")]
    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: AppUserEvent) {
        match event {
            AppUserEvent::Scroll { delta, ctrl_key } => {
                use winit::event::MouseScrollDelta;

                if ctrl_key {
                    self.workspace.update_zoom(delta.y / 32.);
                } else {
                    self.workspace.handle_scroll(MouseScrollDelta::PixelDelta(delta), self.main_modifier);
                }
                self.renderer.request_redraw();
            }
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

            // This is handled in the user_event method
            #[cfg(not(target_arch = "wasm32"))]
            WindowEvent::MouseWheel { delta, .. } => {
                self.workspace.handle_scroll(delta, self.main_modifier);
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
