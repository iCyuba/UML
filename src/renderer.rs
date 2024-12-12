use crate::fonts::FontResource;
use std::num::NonZeroUsize;
use std::sync::Arc;
use vello::kurbo::Affine;
use vello::peniko::{BrushRef, Color, Fill, StyleRef};
use vello::util::{RenderContext, RenderSurface};
use vello::wgpu::{Maintain, PresentMode};
use vello::{AaConfig, Glyph, Renderer, RendererOptions, Scene};
use winit::dpi::{LogicalSize, PhysicalSize};
use winit::event_loop::ActiveEventLoop;
use winit::window::Window;

const WIDTH: u32 = 1024;
const HEIGHT: u32 = 800;

// Code from: https://github.com/linebender/vello/blob/2e2cb1601de7faa85cb3fa87cd03bac9ea10d233/examples/simple/src/main.rs

pub struct WindowRenderer<'s> {
    context: RenderContext,

    renderer: Option<Renderer>,
    surface: Option<RenderSurface<'s>>,

    pub window: Option<Arc<Window>>,
    pub scene: Scene,
}

impl Default for WindowRenderer<'_> {
    fn default() -> Self {
        WindowRenderer {
            context: RenderContext::new(),
            renderer: None,
            surface: None,
            window: None,
            scene: Scene::new(),
        }
    }
}

impl WindowRenderer<'_> {
    pub fn init(&mut self, event_loop: &ActiveEventLoop) {
        if self.surface.is_some() {
            return;
        }

        // Create a new window
        let attr = Window::default_attributes()
            .with_inner_size(LogicalSize::new(WIDTH, HEIGHT))
            .with_resizable(true)
            .with_title("UML Editor");

        let window = Arc::new(event_loop.create_window(attr).unwrap());

        // Create a new surface
        let size = window.inner_size();
        let surface_future = self.context.create_surface(
            window.clone(),
            size.width,
            size.height,
            PresentMode::AutoVsync,
        );
        let surface = pollster::block_on(surface_future).unwrap();

        // Create a new renderer
        let renderer = Renderer::new(
            &self.context.devices[surface.dev_id].device,
            RendererOptions {
                surface_format: Some(surface.format),
                use_cpu: false,
                antialiasing_support: vello::AaSupport::all(),
                num_init_threads: NonZeroUsize::new(1),
            },
        ).unwrap();

        // Save
        self.window = Some(window);
        self.surface = Some(surface);
        self.renderer = Some(renderer);
    }

    pub fn render(&mut self) {
        let surface = self.surface.as_ref().unwrap();
        let renderer = self.renderer.as_mut().unwrap();

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
        renderer.render_to_surface(
            &device_handle.device,
            &device_handle.queue,
            &self.scene,
            &surface_texture,
            &vello::RenderParams {
                base_color: Color::WHITE,
                width,
                height,
                antialiasing_method: AaConfig::Msaa16,
            },
        ).expect("failed to render to surface");

        // Queue the texture to be presented on the surface
        surface_texture.present();
        device_handle.device.poll(Maintain::Poll);
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        let surface = self.surface.as_mut().unwrap();

        self.context.resize_surface(surface, size.width, size.height);
        self.request_redraw();
    }

    pub fn request_redraw(&self) {
        let window = self.window.as_ref().unwrap();
        window.request_redraw();
    }
}

pub fn add_text_to_scene(scene: &mut Scene, text: &str, x: f64, y: f64, size: f32, font: &FontResource) {
    let metrics = font.metrics(size);
    let mut p_x = 0.0;

    scene.draw_glyphs(&font.font)
        .font_size(size)
        .brush(BrushRef::Solid(Color { r: 0, g: 0, b: 0, a: 255 }))
        .transform(Affine::translate((x, y + size as f64)))
        .draw(StyleRef::Fill(Fill::NonZero), text.chars().map(|c| {
            let glyph_id = font.char_map.map(c).unwrap_or_default();
            let x = p_x;

            p_x += metrics.advance_width(glyph_id).unwrap_or_default();

            Glyph {
                id: glyph_id.to_u32(),
                x,
                y: 0.,
            }
        }));
}
