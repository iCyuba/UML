#[cfg(target_arch = "wasm32")]
use crate::app::AppUserEvent;
use crate::presentation::Colors;
use std::boxed::Box;
use std::cell::RefCell;
use std::error::Error;
use std::num::NonZeroUsize;
use std::rc::Rc;
use std::result::Result;
use std::sync::Arc;
use vello::util::{RenderContext, RenderSurface};
use vello::wgpu::{Maintain, PresentMode};
use vello::{AaConfig, RendererOptions, Scene};
use winit::dpi::{LogicalSize, PhysicalSize};
#[cfg(not(target_arch = "wasm32"))]
use winit::event_loop::ActiveEventLoop;
#[cfg(target_arch = "wasm32")]
use winit::event_loop::EventLoop;
use winit::window::{CursorIcon, Theme, Window, WindowAttributes};

use super::{Canvas, Renderer};

const WIDTH: u32 = 1024;
const HEIGHT: u32 = 800;

// Code from: https://github.com/linebender/vello/blob/2e2cb1601de7faa85cb3fa87cd03bac9ea10d233/examples/simple/src/main.rs

pub struct WindowRenderer<'s> {
    // This needs to be shared with the png renderer
    context: Rc<RefCell<RenderContext>>,

    renderer: Option<vello::Renderer>,
    surface: Option<RenderSurface<'s>>,

    // This is only needed on the web, because the re-renders are bound to requestAnimationFrame
    #[cfg(target_arch = "wasm32")]
    resize_on_next_frame: Option<PhysicalSize<u32>>,

    pub window: Option<Arc<Window>>,
    pub canvas: Canvas,
}

impl WindowRenderer<'_> {
    pub fn new(context: Rc<RefCell<RenderContext>>) -> Self {
        WindowRenderer {
            context,
            renderer: None,
            surface: None,
            window: None,
            canvas: Canvas {
                size: (WIDTH, HEIGHT),
                scale: 1.0,
                colors: &Colors::LIGHT,
                scene: Scene::new(),
            },

            #[cfg(target_arch = "wasm32")]
            resize_on_next_frame: None,
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub async fn init(&mut self, event_loop: &ActiveEventLoop) -> Result<(), Box<dyn Error>> {
        if self.surface.is_some() {
            return Ok(());
        }

        // Create a new window
        let window = Arc::new(event_loop.create_window(Self::window_attributes())?);

        // Create a new surface and renderer
        let size = window.inner_size();
        let surface = self
            .context
            .borrow_mut()
            .create_surface(
                window.clone(),
                size.width,
                size.height,
                PresentMode::AutoVsync,
            )
            .await?;

        let renderer = self.create_vello_renderer(&surface);

        // Set the user theme
        if let Some(theme) = window.theme() {
            self.update_theme(theme);
        }

        // Save
        self.window = Some(window);
        self.surface = Some(surface);
        self.renderer = Some(renderer);

        Ok(())
    }

    #[cfg(target_arch = "wasm32")]
    pub async fn init(&mut self, event_loop: &EventLoop<AppUserEvent>) {
        use web_sys::wasm_bindgen::closure::Closure;
        use web_sys::wasm_bindgen::JsCast;
        use winit::platform::web::{WindowAttributesExtWebSys, WindowExtWebSys};

        // Create a new window
        let attributes = Self::window_attributes().with_append(true);

        #[allow(deprecated)]
        let window = Arc::new(event_loop.create_window(attributes).unwrap());
        window.canvas().unwrap().focus().unwrap();

        // Web setup
        let web_window = web_sys::window().unwrap();
        let width = web_window.inner_width().unwrap().as_f64().unwrap();
        let height = web_window.inner_height().unwrap().as_f64().unwrap();
        let device_pixel_ratio = web_window.device_pixel_ratio();

        let size = PhysicalSize::from_logical::<_, f64>((width, height), device_pixel_ratio);
        let _ = window.request_inner_size(size);

        let cloned_window = window.clone();

        // JavaScript resize event
        let resize_cb = Closure::wrap(Box::new(move |_: web_sys::Event| {
            let web_window = web_sys::window().unwrap();

            let width = web_window.inner_width().unwrap().as_f64().unwrap();
            let height = web_window.inner_height().unwrap().as_f64().unwrap();
            let device_pixel_ratio = web_window.device_pixel_ratio();

            let size =
                PhysicalSize::<u32>::from_logical::<_, f64>((width, height), device_pixel_ratio);
            let _ = cloned_window.request_inner_size(size);
        }) as Box<dyn FnMut(_)>);

        web_window
            .add_event_listener_with_callback("resize", &resize_cb.as_ref().unchecked_ref())
            .unwrap();

        resize_cb.forget();

        // Create a new surface and renderer
        let surface = self
            .context
            .borrow_mut()
            .create_surface(
                window.clone(),
                size.width,
                size.height,
                PresentMode::AutoVsync,
            )
            .await
            .unwrap();
        let renderer = self.create_vello_renderer(&surface);

        // Set the user theme
        if let Some(theme) = window.theme() {
            self.update_theme(theme);
        }

        // Set the window's scale
        self.canvas.scale = window.scale_factor();

        // Save
        self.window = Some(window);
        self.surface = Some(surface);
        self.renderer = Some(renderer);
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        let surface = self.surface.as_mut().unwrap();

        self.context
            .borrow_mut()
            .resize_surface(surface, size.width.max(1), size.height.max(1));

        self.canvas.size = (size.width, size.height);

        self.request_redraw();
    }

    #[cfg(target_arch = "wasm32")]
    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.resize_on_next_frame = Some(size);
        self.canvas.size = (size.width, size.height);
    }

    pub fn set_scale(&mut self, scale: f64) {
        self.canvas.scale = scale;
        self.request_redraw();
    }

    pub fn request_redraw(&self) {
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
    }

    pub fn update_theme(&mut self, theme: Theme) {
        match theme {
            Theme::Light => self.canvas.colors = &Colors::LIGHT,
            Theme::Dark => self.canvas.colors = &Colors::DARK,
        }

        self.request_redraw();
    }

    pub fn set_cursor(&self, cursor: CursorIcon) {
        let window = self.window.as_ref().unwrap();
        window.set_cursor(cursor);
    }

    fn window_attributes() -> WindowAttributes {
        Window::default_attributes()
            .with_inner_size(LogicalSize::new(WIDTH, HEIGHT))
            .with_resizable(true)
            .with_title("UML Editor")
    }

    fn create_vello_renderer(&self, surface: &RenderSurface) -> vello::Renderer {
        vello::Renderer::new(
            &self.context.borrow().devices[surface.dev_id].device,
            RendererOptions {
                surface_format: Some(surface.format),
                use_cpu: false,
                antialiasing_support: vello::AaSupport::all(),
                num_init_threads: NonZeroUsize::new(1),
            },
        )
        .unwrap()
    }
}

impl Renderer for WindowRenderer<'_> {
    // There's no output, it renders to the window directly
    type RenderOutput = ();

    fn canvas(&mut self) -> &mut super::Canvas {
        &mut self.canvas
    }

    fn render(&mut self) -> Result<(), Box<dyn Error>> {
        let surface = self.surface.as_mut().ok_or("surface not initialized")?;
        let renderer = self.renderer.as_mut().ok_or("renderer not initialized")?;

        #[cfg(target_arch = "wasm32")]
        if let Some(size) = self.resize_on_next_frame.take() {
            self.context
                .borrow()
                .resize_surface(surface, size.width.max(1), size.height.max(1));
        }

        // Get the window size
        let width = surface.config.width;
        let height = surface.config.height;

        // Get a handle to the device
        let device_handle = &self.context.borrow().devices[surface.dev_id];

        // Get the surface's texture
        let surface_texture = surface
            .surface
            .get_current_texture()
            .expect("failed to get current texture");

        // Render to the surface's texture
        renderer
            .render_to_surface(
                &device_handle.device,
                &device_handle.queue,
                &self.canvas.scene,
                &surface_texture,
                &vello::RenderParams {
                    base_color: self.canvas.colors.workspace_background,
                    width,
                    height,
                    antialiasing_method: AaConfig::Msaa16,
                },
            )
            .expect("failed to render to surface");

        // Queue the texture to be presented on the surface
        surface_texture.present();
        device_handle.device.poll(Maintain::Poll);

        Ok(())
    }
}
