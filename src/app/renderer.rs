#[cfg(target_arch = "wasm32")]
use crate::app::AppUserEvent;
use crate::presentation::{Colors, FontResource};
use std::num::NonZeroUsize;
use std::sync::Arc;
use vello::kurbo::Affine;
use vello::peniko::{BrushRef, Fill, StyleRef};
use vello::util::{RenderContext, RenderSurface};
use vello::wgpu::{Maintain, PresentMode};
use vello::{AaConfig, Glyph, RendererOptions, Scene};
use winit::dpi::{LogicalSize, PhysicalSize};
#[cfg(not(target_arch = "wasm32"))]
use winit::event_loop::ActiveEventLoop;
#[cfg(target_arch = "wasm32")]
use winit::event_loop::EventLoop;
use winit::window::{Theme, Window, WindowAttributes};

const WIDTH: u32 = 1024;
const HEIGHT: u32 = 800;

// Code from: https://github.com/linebender/vello/blob/2e2cb1601de7faa85cb3fa87cd03bac9ea10d233/examples/simple/src/main.rs

pub struct Renderer<'s> {
    context: RenderContext,

    renderer: Option<vello::Renderer>,
    surface: Option<RenderSurface<'s>>,

    // This is only needed on the web, because the re-renders are bound to requestAnimationFrame
    #[cfg(target_arch = "wasm32")]
    resize_on_next_frame: Option<PhysicalSize<u32>>,

    pub window: Option<Arc<Window>>,
    pub scene: Scene,

    pub colors: &'static Colors,
}

impl Default for Renderer<'_> {
    fn default() -> Self {
        Renderer {
            context: RenderContext::new(),
            renderer: None,
            surface: None,
            window: None,
            scene: Scene::new(),
            colors: &Colors::LIGHT,

            #[cfg(target_arch = "wasm32")]
            resize_on_next_frame: None,
        }
    }
}

impl Renderer<'_> {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn init(&mut self, event_loop: &ActiveEventLoop) {
        if self.surface.is_some() {
            return;
        }

        // Create a new window
        let window = Arc::new(event_loop.create_window(Self::window_attributes()).unwrap());

        // Create a new surface and renderer
        let size = window.inner_size();
        let surface_future = self.context.create_surface(
            window.clone(),
            size.width,
            size.height,
            PresentMode::AutoVsync,
        );
        let surface = pollster::block_on(surface_future).unwrap();
        let renderer = self.create_vello_renderer(&surface);

        // Set the user theme
        if let Some(theme) = window.theme() {
            self.update_theme(theme);
        }

        // Save
        self.window = Some(window);
        self.surface = Some(surface);
        self.renderer = Some(renderer);
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

        // Save
        self.window = Some(window);
        self.surface = Some(surface);
        self.renderer = Some(renderer);
    }

    pub fn render(&mut self) {
        let surface = self.surface.as_mut().unwrap();
        let renderer = self.renderer.as_mut().unwrap();

        #[cfg(target_arch = "wasm32")]
        if let Some(size) = self.resize_on_next_frame.take() {
            self.context
                .resize_surface(surface, size.width, size.height);
        }

        // Get the window size
        let width = surface.config.width;
        let height = surface.config.height;

        // Get a handle to the device
        let device_handle = &self.context.devices[surface.dev_id];

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
                &self.scene,
                &surface_texture,
                &vello::RenderParams {
                    base_color: self.colors.workspace_background,
                    width,
                    height,
                    antialiasing_method: AaConfig::Msaa16,
                },
            )
            .expect("failed to render to surface");

        // Queue the texture to be presented on the surface
        surface_texture.present();
        device_handle.device.poll(Maintain::Poll);
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        let surface = self.surface.as_mut().unwrap();

        self.context
            .resize_surface(surface, size.width, size.height);
        self.request_redraw();
    }

    #[cfg(target_arch = "wasm32")]
    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.resize_on_next_frame = Some(size);
    }

    pub fn request_redraw(&self) {
        let window = self.window.as_ref().unwrap();
        window.request_redraw();
    }

    pub fn update_theme(&mut self, theme: Theme) {
        match theme {
            Theme::Light => self.colors = &Colors::LIGHT,
            Theme::Dark => self.colors = &Colors::DARK,
        }
    }

    pub fn size(&self) -> PhysicalSize<u32> {
        self.window.as_ref().unwrap().inner_size()
    }

    pub fn scale(&self) -> f64 {
        self.window.as_ref().unwrap().scale_factor()
    }

    fn window_attributes() -> WindowAttributes {
        Window::default_attributes()
            .with_inner_size(LogicalSize::new(WIDTH, HEIGHT))
            .with_resizable(true)
            .with_title("UML Editor")
    }

    fn create_vello_renderer(&self, surface: &RenderSurface) -> vello::Renderer {
        vello::Renderer::new(
            &self.context.devices[surface.dev_id].device,
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

pub fn add_text_to_scene<'a>(
    scene: &'a mut Scene,
    text: &'a str,
    x: f64,
    y: f64,
    size: f32,
    font: &'a FontResource,
    brush: impl Into<BrushRef<'a>>,
) {
    let metrics = font.metrics(size);
    let mut p_x = 0.0;

    scene
        .draw_glyphs(&font.font)
        .font_size(size)
        .brush(brush)
        .transform(Affine::translate((x, y + size as f64)))
        .draw(
            StyleRef::Fill(Fill::NonZero),
            text.chars().map(|c| {
                let glyph_id = font.char_map.map(c).unwrap_or_default();
                let x = p_x;

                p_x += metrics.advance_width(glyph_id).unwrap_or_default();

                Glyph {
                    id: glyph_id.to_u32(),
                    x,
                    y: 0.,
                }
            }),
        );
}
