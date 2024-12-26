use super::{EventTarget, Renderer, State, Tree};
use crate::geometry::{Point, Vec2};
use winit::application::ApplicationHandler;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoopProxy};

#[derive(Debug)]
pub enum AppUserEvent {
    #[cfg(target_arch = "wasm32")]
    Scroll { delta: Vec2, ctrl_key: bool },
}

pub struct App<'s> {
    #[allow(dead_code)] // Unused on the desktop app rn
    event_loop: EventLoopProxy<AppUserEvent>,

    pub renderer: Renderer<'s>,
    pub state: State,

    pub tree: Tree,
}

impl App<'_> {
    pub fn new(event_loop: EventLoopProxy<AppUserEvent>) -> Self {
        App {
            event_loop,

            renderer: Default::default(),
            state: Default::default(),

            tree: Tree::new(),
        }
    }

    pub fn redraw(&mut self) {
        self.renderer.scene.reset();

        // Flex layout
        let size = self.renderer.size();
        let scale = self.renderer.scale() as f32;

        self.tree.compute_layout(size, scale);

        self.tree.update(&mut self.state);
        self.tree.render(&mut self.renderer, &self.state);

        // Draw the scene onto the screen
        self.renderer.render();
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
        let use_super = window
            .navigator()
            .user_agent()
            .unwrap()
            .to_lowercase()
            .contains("mac");

        self.state.use_super = use_super;

        // Setup a better scroll handler
        let proxy = self.event_loop.clone();
        let closure = Closure::wrap(Box::new(move |event: web_sys::WheelEvent| {
            proxy
                .send_event(AppUserEvent::Scroll {
                    delta: -Vec2 {
                        x: event.delta_x(),
                        y: event.delta_y(),
                    },
                    ctrl_key: event.ctrl_key() || (use_super && event.meta_key()),
                })
                .unwrap();
        }) as Box<dyn FnMut(_)>);

        window
            .add_event_listener_with_callback("wheel", &closure.as_ref().unchecked_ref())
            .unwrap();
        closure.forget();
    }

    #[cfg(target_arch = "wasm32")]
    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: AppUserEvent) {
        match event {
            AppUserEvent::Scroll { delta, ctrl_key } => {
                self.viewport
                    .on_scroll(&mut self.state, delta, false, ctrl_key, false);
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
            WindowEvent::RedrawRequested => self.redraw(),

            // This is handled in the user_event method
            #[cfg(not(target_arch = "wasm32"))]
            WindowEvent::MouseWheel { delta, .. } => {
                use winit::dpi::PhysicalPosition;
                use winit::event::MouseScrollDelta;

                let mouse = match delta {
                    MouseScrollDelta::LineDelta(_, _) => true,
                    MouseScrollDelta::PixelDelta(_) => false,
                };

                let delta = match delta {
                    MouseScrollDelta::LineDelta(x, y) => Vec2 {
                        x: x as f64 * 64.,
                        y: y as f64 * 64.,
                    },

                    MouseScrollDelta::PixelDelta(PhysicalPosition { x, y }) => Vec2 { x, y },
                };

                let zoom = self.state.main_modifier();
                let reverse = self.state.modifiers.shift_key();

                self.tree
                    .on_wheel(&mut self.state, delta, mouse, zoom, reverse);
            }

            WindowEvent::PinchGesture { delta, .. } => self.tree.on_wheel(
                &mut self.state,
                Vec2 {
                    x: 0.,
                    y: delta * 256.,
                },
                false,
                true,
                false,
            ),

            WindowEvent::CursorMoved { position, .. } => {
                let cursor = Point::from(position);

                self.tree.on_mousemove(&mut self.state, cursor);
                self.state.cursor = cursor;
            }

            WindowEvent::ThemeChanged(theme) => {
                self.renderer.update_theme(theme);
                self.state.redraw = true;
            }

            WindowEvent::MouseInput { state, button, .. } => {
                if state == ElementState::Pressed {
                    self.state.mouse_buttons.insert(button);
                    self.tree.on_mousedown(&mut self.state, button);
                } else {
                    self.state.mouse_buttons.remove(&button);
                    self.tree.on_mouseup(&mut self.state, button);
                }
            }

            WindowEvent::KeyboardInput { event, .. } => {
                if event.state == ElementState::Pressed {
                    self.state.keys.insert(event.logical_key);
                } else {
                    self.state.keys.remove(&event.logical_key);
                }
            }

            WindowEvent::ModifiersChanged(modifiers) => self.state.modifiers = modifiers.state(),

            _ => {}
        }

        // Request a new frame if needed
        if self.state.redraw {
            self.renderer.request_redraw();
            self.state.redraw = false;
        }
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        // Do nothing
    }
}
