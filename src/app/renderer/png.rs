use super::{Canvas, Renderer};
use crate::presentation::Colors;
use std::{cell::RefCell, num::NonZeroUsize, rc::Rc};
use vello::{
    util::{block_on_wgpu, RenderContext},
    wgpu::{
        self, BufferDescriptor, BufferUsages, Extent3d, ImageCopyBuffer, ImageDataLayout, MapMode,
        TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
    },
    AaConfig, RenderParams, RendererOptions, Scene,
};

const WIDTH: u32 = 1200;
const HEIGHT: u32 = 800;

// Heavily inspired by: https://github.com/linebender/vello/blob/8a84a4abf7aaabdb7de82e3d9d86ed427ad21638/examples/headless/src/main.rs

pub struct PngRenderer {
    // This needs to be shared with the window renderer
    context: Rc<RefCell<RenderContext>>,

    texture: Option<wgpu::Texture>,
    texture_view: Option<wgpu::TextureView>,

    renderer: vello::Renderer,
    device: usize,

    canvas: Canvas,
}

impl PngRenderer {
    /// Create a new PNG renderer.
    ///
    /// Mutably borrows the render context for the duration of the function.
    pub async fn new(context: Rc<RefCell<RenderContext>>) -> Result<Self, vello::Error> {
        let device = context
            .borrow_mut()
            .device(None)
            .await
            .ok_or(vello::Error::NoCompatibleDevice)?;

        let renderer = vello::Renderer::new(
            &context.borrow().devices[device].device,
            RendererOptions {
                surface_format: None,
                use_cpu: false,
                antialiasing_support: vello::AaSupport::all(),
                num_init_threads: NonZeroUsize::new(1),
            },
        )?;

        let canvas = Canvas {
            size: (WIDTH, HEIGHT),
            scale: 1.0,
            colors: &Colors::LIGHT,
            scene: Scene::new(),
        };

        Ok(Self {
            context,
            texture: None,
            texture_view: None,
            renderer,
            device,
            canvas,
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if self.canvas.size.0 == width && self.canvas.size.1 == height {
            return;
        }

        self.canvas.size = (width, height);

        // Reset the reusables
        self.texture = None;
        self.texture_view = None;
    }
}

impl Renderer for PngRenderer {
    type RenderOutput = Vec<u8>;

    fn canvas(&mut self) -> &mut Canvas {
        &mut self.canvas
    }

    // Immutably borrows the context to get the device and queue
    // Blocks the current thread until the gpu operations are done
    fn render(&mut self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        #[cfg(target_arch = "wasm32")]
        panic!("PNG renderer is not supported in the browser");

        let context = self.context.borrow();

        let device = &context.devices[self.device].device;
        let queue = &context.devices[self.device].queue;

        let (width, height) = self.canvas.size;
        let size = Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        // Create the texture and view if they don't exist
        let texture = self.texture.get_or_insert_with(|| {
            device.create_texture(&TextureDescriptor {
                label: None,
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba8Unorm,
                usage: TextureUsages::STORAGE_BINDING | TextureUsages::COPY_SRC,
                view_formats: &[],
            })
        });

        let texture_view = self
            .texture_view
            .get_or_insert_with(|| texture.create_view(&Default::default()));

        // Render the scene to the texture
        self.renderer.render_to_texture(
            device,
            queue,
            &self.canvas.scene,
            texture_view,
            &RenderParams {
                base_color: Colors::LIGHT.workspace_background,
                width,
                height,
                antialiasing_method: AaConfig::Msaa16,
            },
        )?;

        // Create a new buffer to read the texture

        // Align the width to the next multiple of 256
        let padded_byte_width = (width * 4).next_multiple_of(256);
        let buffer_size = padded_byte_width as u64 * height as u64;

        let buffer = device.create_buffer(&BufferDescriptor {
            label: None,
            size: buffer_size,
            usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Copy the texture to the buffer
        let mut encoder = device.create_command_encoder(&Default::default());
        encoder.copy_texture_to_buffer(
            texture.as_image_copy(),
            ImageCopyBuffer {
                buffer: &buffer,
                layout: ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_byte_width),
                    rows_per_image: None,
                },
            },
            size,
        );

        queue.submit([encoder.finish()]);

        // Read the buffer
        let buffer_slice = buffer.slice(..);

        let (sender, receiver) = futures_intrusive::channel::shared::oneshot_channel();
        buffer_slice.map_async(MapMode::Read, move |v| sender.send(v).unwrap());

        block_on_wgpu(device, receiver.receive()).unwrap().unwrap();

        let data = buffer_slice.get_mapped_range();

        // Unpad the data
        let mut result_unpadded = Vec::<u8>::with_capacity((width * height * 4) as usize);
        for row in 0..height {
            let start = (row * padded_byte_width) as usize;
            result_unpadded.extend(&data[start..start + (width * 4) as usize]);
        }

        // Encode the image into a vec
        let mut png_data = Vec::new();

        let mut encoder = png::Encoder::new(&mut png_data, width, height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);

        let mut writer = encoder.write_header()?;
        writer.write_image_data(&result_unpadded)?;
        writer.finish()?;

        Ok(png_data)
    }
}
