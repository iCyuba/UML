use super::{context, ctx, renderer, EventTarget, Renderer, State, Tree};
use crate::{
    app::event_target::WheelEvent,
    data::{entity::EntityType, project::TokenVec, Project},
    elements::toolbox_item::Tool,
    geometry::{Point, Vec2},
};
use ogrim::xml;
#[cfg(not(target_arch = "wasm32"))]
use rfd::FileDialog;
use std::{cell::RefCell, fmt, io::Write, rc::Rc};
use winit::{
    application::ApplicationHandler,
    event::{ElementState, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoopProxy},
    window::CursorIcon,
};
use zip::write::FileOptions;
use zip::ZipWriter;

pub enum AppUserEvent {
    #[cfg(target_arch = "wasm32")]
    Scroll(WheelEvent),
    #[cfg(target_arch = "wasm32")]
    FileLoaded(Vec<u8>, String),

    RequestRedraw,
    RequestCursorUpdate,
    RequestTooltipUpdate,
    ModifyTree(Box<dyn FnOnce(&mut Tree, &mut context::EventContext) + 'static>),
    Screenshot,
    Save,
    Load,
    Export,
    SetTool(Tool),
}

impl fmt::Debug for AppUserEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            #[cfg(target_arch = "wasm32")]
            AppUserEvent::Scroll(ev) => f.debug_tuple("Scroll").field(ev).finish(),
            #[cfg(target_arch = "wasm32")]
            AppUserEvent::FileLoaded(_, _) => f.write_str("FileLoaded"),

            AppUserEvent::RequestRedraw => f.write_str("RequestRedraw"),
            AppUserEvent::RequestCursorUpdate => f.write_str("RequestCursorUpdate"),
            AppUserEvent::RequestTooltipUpdate => f.write_str("RequestTooltipUpdate"),
            AppUserEvent::ModifyTree(_) => f.write_str("ModifyTree"),
            AppUserEvent::Screenshot => f.write_str("Screenshot"),
            AppUserEvent::Save => f.write_str("Save"),
            AppUserEvent::Export => f.write_str("Export"),
            AppUserEvent::Load => f.write_str("Load"),
            AppUserEvent::SetTool(tool) => f.debug_tuple("SetTool").field(tool).finish(),
        }
    }
}

pub struct App<'s> {
    pub window: renderer::WindowRenderer<'s>,
    #[cfg(not(target_arch = "wasm32"))]
    pub png: renderer::PngRenderer,

    pub state: State,

    pub tree: Tree,
    pub project: Project,
}

impl App<'_> {
    pub async fn new(event_loop: EventLoopProxy<AppUserEvent>) -> Self {
        let vello_render_context = Rc::new(RefCell::new(vello::util::RenderContext::new()));

        let mut window = renderer::WindowRenderer::new(vello_render_context.clone());

        #[cfg(not(target_arch = "wasm32"))]
        let png = renderer::PngRenderer::new(vello_render_context)
            .await
            .expect("Couldn't create PNG renderer");

        let mut state = State::new(event_loop);
        let mut project = Project::new(String::new());

        // Can't use the macro here, because App doesn't exist yet
        let tree = Tree::new(&mut context::EventContext {
            project: &mut project,
            state: &mut state,
            c: window.canvas(),
        });

        App {
            window,
            #[cfg(not(target_arch = "wasm32"))]
            png,

            state,

            tree,
            project,
        }
    }

    pub fn redraw(&mut self) {
        self.window.canvas.reset();

        // Flex layout
        self.tree.update(ctx!(self));
        self.tree.render(ctx!(self));

        // Draw the scene onto the screen
        self.window.render().unwrap();
    }

    pub fn load_project(&mut self, data: Vec<u8>, name: &str) {
        let extension = name.split('.').last().unwrap_or_default();

        let Some(project) = (match extension {
            "json" => serde_json::from_slice(&data).ok(),
            _ => postcard::from_bytes(&data).ok(),
        }) else {
            return;
        };

        self.project = project;
        self.window.request_redraw();
    }

    pub fn update_cursor(&mut self) {
        let cursor = self.tree.cursor(ctx!(self)).unwrap_or(CursorIcon::Default);
        self.window.set_cursor(cursor);
    }
}

impl ApplicationHandler<AppUserEvent> for App<'_> {
    #[cfg(not(target_arch = "wasm32"))]
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.window.is_none() {
            pollster::block_on(self.window.init(event_loop)).unwrap();
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {
        use crate::web;

        // macOS keybind setup + scroll event + file picker
        self.state.use_super = web::use_super();
        web::setup_scroll_event(self.state.event_loop.clone());
        web::setup_file_picker(self.state.event_loop.clone());
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: AppUserEvent) {
        macro_rules! ctx {
            () => {
                super::ctx!(self)
            };
        }

        match event {
            #[cfg(target_arch = "wasm32")]
            AppUserEvent::Scroll(ev) => {
                self.tree.on_wheel(ctx!(), ev);
                self.window.request_redraw();
            }
            #[cfg(target_arch = "wasm32")]
            AppUserEvent::FileLoaded(data, name) => {
                self.load_project(data, &name);
            }

            AppUserEvent::RequestRedraw => self.window.request_redraw(),
            AppUserEvent::RequestCursorUpdate => self.update_cursor(),
            AppUserEvent::RequestTooltipUpdate => {
                self.state.tooltip_state = self.tree.tooltip(ctx!());
                self.window.request_redraw();
            }
            AppUserEvent::ModifyTree(f) => {
                f(&mut self.tree, ctx!());
            }
            #[cfg(not(target_arch = "wasm32"))]
            AppUserEvent::Screenshot => {
                let (width, height) = self.window.canvas.size();
                let scale = self.window.canvas.scale();

                let width = (width as f64 / scale) as u32;
                let height = (height as f64 / scale) as u32;
                self.png.resize(width, height);

                let canvas = self.png.canvas();
                canvas.reset();

                self.tree.render(&mut context::RenderContext {
                    c: canvas,
                    project: &self.project,
                    state: &self.state,
                });

                let image = self.png.render().unwrap();

                // Save the image
                let Some(path) = FileDialog::new()
                    .add_filter("png", &["png"])
                    .set_file_name("screenshot")
                    .save_file()
                else {
                    return;
                };

                _ = std::fs::write(path, image);
            }
            #[cfg(target_arch = "wasm32")]
            AppUserEvent::Screenshot => {
                crate::web::screenshot(self.window.window.as_ref().unwrap());
            }
            #[cfg(not(target_arch = "wasm32"))]
            AppUserEvent::Save => {
                let Some(path) = FileDialog::new()
                    .add_filter("binary", &["bin"])
                    .add_filter("json", &["json"])
                    .set_file_name(self.project.get_sanitized_name().to_lowercase() + ".bin")
                    .save_file()
                else {
                    return;
                };

                let extension = path.extension().and_then(|ext| ext.to_str());

                let data = match extension {
                    Some("json") => serde_json::to_vec_pretty(&self.project).unwrap(),
                    _ => postcard::to_stdvec(&self.project).unwrap(),
                };

                _ = std::fs::write(path, data);
            }
            #[cfg(target_arch = "wasm32")]
            AppUserEvent::Save => {
                let json = self.state.modifiers.shift_key();

                let mut name = self.project.get_sanitized_name().to_lowercase();
                let data = if json {
                    name += ".json";

                    serde_json::to_vec_pretty(&self.project).unwrap()
                } else {
                    name += ".bin";

                    postcard::to_stdvec(&self.project).unwrap()
                };

                crate::web::download_bytes(&data, &name);
            }
            #[cfg(not(target_arch = "wasm32"))]
            AppUserEvent::Load => {
                let Some(path) = FileDialog::new()
                    .add_filter("binary", &["bin"])
                    .add_filter("json", &["json"])
                    .set_directory("data")
                    .set_file_name("project.bin")
                    .pick_file()
                else {
                    return;
                };

                if let Ok(data) = std::fs::read(&path) {
                    self.load_project(data, path.file_name().unwrap().to_str().unwrap());
                }
            }
            #[cfg(target_arch = "wasm32")]
            AppUserEvent::Load => {
                crate::web::open_file_picker();
            }
            AppUserEvent::SetTool(tool) => {
                self.state.tool = tool;

                // Interfaces can't have parent relations, so deselect the entity if the tool parent is used
                if self
                    .state
                    .selected_entity
                    .and_then(|e| self.project.entities.get(e))
                    .is_some_and(|e| e.entity_type == EntityType::Interface)
                    && tool == Tool::Parent
                {
                    self.state.selected_entity = None;
                }

                self.update_cursor();
                self.window.request_redraw();
            }
            AppUserEvent::Export => {
                let name = self.project.get_sanitized_name();

                #[allow(unused_mut)]
                let mut file;
                let mut zip;

                #[cfg(not(target_arch = "wasm32"))]
                {
                    let Some(path) = FileDialog::new()
                        .add_filter("zip", &["zip"])
                        .set_file_name(name.clone() + ".zip")
                        .save_file()
                    else {
                        return;
                    };

                    file = std::fs::File::create(&path).unwrap();
                    zip = ZipWriter::new(file);
                }
                #[cfg(target_arch = "wasm32")]
                {
                    file = Vec::<u8>::new();
                    zip = ZipWriter::new(std::io::Cursor::new(&mut file));
                }

                let options: FileOptions<()> = FileOptions::default()
                    .compression_method(zip::CompressionMethod::Deflated)
                    .unix_permissions(0o755);

                zip.add_directory(&name, options).unwrap();

                // Write project file
                let proj = xml!(
                    <?xml version="1.0"?>
                    <Project Sdk="Microsoft.NET.Sdk">
                      <PropertyGroup>
                        <TargetFramework>"net8.0"</>
                        <ImplicitUsings>"enable"</>
                        <Nullable>"enable"</>
                      </>
                    </>
                );

                let project_file = format!("{name}/{name}.csproj");
                zip.start_file(project_file.clone(), options).unwrap();
                writeln!(zip, "{}", proj.as_str()).unwrap();

                // Write solution file
                let sln = xml!(
                    <?xml version="1.0"?>
                    <Solution>
                      <Project Path={project_file} />
                    </>
                );

                zip.start_file(format!("{name}.slnx"), options).unwrap();
                writeln!(zip, "{}", sln.as_str()).unwrap();

                // Write entities
                for (key, entity) in &self.project.entities {
                    let file_name = format!("{name}/{}.cs", self.project.sanitize(&entity.name));
                    zip.start_file(file_name, options).unwrap();
                    let tokens = self.project.entity_to_token(key);

                    writeln!(zip, "{}", TokenVec(tokens)).unwrap();
                }

                zip.finish().unwrap();

                #[cfg(target_arch = "wasm32")]
                {
                    crate::web::download_bytes(&file, &(name + ".zip"));
                }
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        macro_rules! ctx {
            () => {
                super::ctx!(self)
            };
        }

        let Some(window) = self.window.window.as_ref() else {
            return;
        };

        if window.id() != window_id {
            return;
        }

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => self.window.resize(size),
            WindowEvent::RedrawRequested => self.redraw(),
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                self.window.set_scale(scale_factor)
            }

            // This is handled in the user_event method
            #[cfg(not(target_arch = "wasm32"))]
            WindowEvent::MouseWheel { delta, .. } => {
                use winit::dpi::PhysicalPosition;
                use winit::event::MouseScrollDelta;

                let mouse = match delta {
                    MouseScrollDelta::LineDelta(_, _) => true,
                    MouseScrollDelta::PixelDelta(_) => false,
                };

                let scale = self.window.canvas.scale();
                let delta = match delta {
                    MouseScrollDelta::LineDelta(x, y) => Vec2 {
                        x: x as f64 * 64.,
                        y: y as f64 * 64.,
                    },

                    // Pretty sure macOS scales up the delta
                    MouseScrollDelta::PixelDelta(PhysicalPosition { x, y }) => Vec2 {
                        x: x / scale,
                        y: y / scale,
                    },
                };

                let zoom = self.state.main_modifier();
                let reverse = self.state.modifiers.shift_key();

                self.tree.on_wheel(
                    ctx!(),
                    WheelEvent {
                        delta,
                        mouse,
                        zoom,
                        reverse,
                    },
                );
            }

            WindowEvent::PinchGesture { delta, .. } => {
                self.tree.on_wheel(
                    ctx!(),
                    WheelEvent {
                        delta: Vec2 {
                            x: 0.,
                            y: delta * 128.,
                        },
                        mouse: false,
                        zoom: true,
                        reverse: false,
                    },
                );
            }

            WindowEvent::CursorMoved { position, .. } => {
                // Scale down the cursor position
                let cursor = Point::from(position) / self.window.canvas.scale();

                self.tree.on_mousemove(ctx!(), cursor);
                self.state.cursor = cursor;
            }

            WindowEvent::CursorLeft { .. } => {
                self.tree.on_mouseleave(ctx!());
            }

            WindowEvent::ThemeChanged(theme) => {
                self.window.update_theme(theme); // This automatically requests a redraw
            }

            WindowEvent::MouseInput { state, button, .. } => {
                if state == ElementState::Pressed {
                    self.state.mouse_buttons.insert(button);
                    self.tree.on_mousedown(ctx!(), button);
                } else {
                    self.state.mouse_buttons.remove(&button);
                    self.tree.on_mouseup(ctx!(), button);
                }
            }

            WindowEvent::KeyboardInput { event, .. } => {
                if event.state == ElementState::Pressed {
                    self.state.keys.insert(event.logical_key.clone());
                    self.tree.on_keydown(ctx!(), event);
                } else {
                    self.state.keys.remove(&event.logical_key);
                    self.tree.on_keyup(ctx!(), event);
                }
            }

            WindowEvent::ModifiersChanged(modifiers) => self.state.modifiers = modifiers.state(),

            _ => {}
        }
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        // Do nothing
    }
}
