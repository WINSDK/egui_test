use std::{collections::BTreeMap, time::Instant};

use egui::{Button, CentralPanel, FontFamily, FontId, RichText, TextStyle};
use wgpu_backend::{RenderPass, ScreenDescriptor};
use winit::event::Event::*;
use winit_backend::{Platform, PlatformDescriptor};

use winit::event_loop::ControlFlow;

mod icons;
mod style;

const INITIAL_WIDTH: u32 = 1300;
const INITIAL_HEIGHT: u32 = 900;

/// A custom event type for the winit app.
enum Event {
    RequestRedraw,
}

/// This is the repaint signal type that egui needs for requesting a repaint from another thread.
/// It sends the custom RequestRedraw event to the winit event loop.
struct ExampleRepaintSignal(std::sync::Mutex<winit::event_loop::EventLoopProxy<Event>>);

impl epi::backend::RepaintSignal for ExampleRepaintSignal {
    fn request_repaint(&self) {
        self.0.lock().unwrap().send_event(Event::RequestRedraw).ok();
    }
}

#[derive(PartialEq)]
enum TabKind {
    Source(String),
    Listing(usize),
}

type Title = &'static str;

struct Buffers {
    inner: BTreeMap<Title, TabKind>,
}

impl Buffers {
    fn has_multiple_tabs(&self) -> bool {
        self.inner.len() != 1
    }
}

impl egui_dock::TabViewer for Buffers {
    type Tab = Title;

    fn ui(&mut self, ui: &mut egui::Ui, title: &mut Self::Tab) {
        match self.inner.get(title) {
            Some(TabKind::Source(src)) => ui.label(src),
            Some(TabKind::Listing(id)) => ui.label(id.to_string()),
            _ => return,
        };
    }

    fn title(&mut self, title: &mut Self::Tab) -> egui::WidgetText {
        (*title).into()
    }

    fn on_close(&mut self, tab: &mut Self::Tab) -> bool {
        if self.inner.len() == 1 {
            false
        } else {
            self.inner.remove(tab);
            true
        }
    }
}

/// A simple egui + wgpu + winit based example.
fn main() {
    let event_loop = winit::event_loop::EventLoopBuilder::<Event>::with_user_event().build();
    let window = winit::window::WindowBuilder::new()
        .with_decorations(true)
        .with_resizable(true)
        .with_transparent(false)
        .with_title("egui-wgpu_winit example")
        .with_inner_size(winit::dpi::PhysicalSize {
            width: INITIAL_WIDTH,
            height: INITIAL_HEIGHT,
        })
        .build(&event_loop)
        .unwrap();

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::PRIMARY,
        dx12_shader_compiler: wgpu::Dx12Compiler::Fxc,
    });

    let surface = unsafe { instance.create_surface(&window).unwrap() };

    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: Some(&surface),
        force_fallback_adapter: false,
    }))
    .unwrap();

    let (device, queue) = pollster::block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            features: wgpu::Features::default(),
            limits: wgpu::Limits::default(),
            label: None,
        },
        None,
    ))
    .unwrap();

    let size = window.inner_size();
    let surface_capabilities = surface.get_capabilities(&adapter);
    let alpha_mode = surface_capabilities.alpha_modes[0];
    let surface_format = {
        let default_format = surface_capabilities.formats[0];

        surface_capabilities
            .formats
            .into_iter()
            .find(wgpu::TextureFormat::is_srgb)
            .unwrap_or(default_format)
    };

    let mut surface_config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width: size.width as u32,
        height: size.height as u32,
        present_mode: wgpu::PresentMode::Fifo,
        alpha_mode,
        view_formats: Vec::new(),
    };

    surface.configure(&device, &surface_config);

    let style = style::Style::default();
    let dock_style = style.dock();

    let mut egui_style = style.egui();

    egui_style.text_styles = [
        (TextStyle::Heading, FontId::new(25.0, FontFamily::Monospace)),
        (
            TextStyle::Name("Heading2".into()),
            FontId::new(25.0, FontFamily::Monospace),
        ),
        (
            TextStyle::Name("Context".into()),
            FontId::new(23.0, FontFamily::Monospace),
        ),
        (TextStyle::Body, FontId::new(18.0, FontFamily::Monospace)),
        (
            TextStyle::Monospace,
            FontId::new(14.0, FontFamily::Monospace),
        ),
        (TextStyle::Button, FontId::new(14.0, FontFamily::Monospace)),
        (TextStyle::Small, FontId::new(10.0, FontFamily::Monospace)),
    ]
    .into();

    // We use the egui_winit_platform crate as the platform.
    let mut platform = Platform::new(PlatformDescriptor {
        physical_width: size.width as u32,
        physical_height: size.height as u32,
        scale_factor: window.scale_factor(),
        style: egui_style,
    });

    // We use the egui_wgpu_backend crate as the render backend
    let mut egui_rpass = RenderPass::new(&device, surface_format, 1);

    let source_title = icon!(EMBED2, "Source");
    let disass_title = icon!(PARAGRAPH_LEFT, "Disassembly");

    let buffers = BTreeMap::from([
        (source_title, TabKind::Listing(1600)),
        (
            disass_title,
            TabKind::Source(String::from("line 1\nline 2\nline 3")),
        ),
    ]);

    let mut buffers = Buffers { inner: buffers };

    // init tab tree
    let mut tree = egui_dock::tree::Tree::new(vec![source_title, disass_title]);
    tree.set_focused_node(egui_dock::NodeIndex::root());

    let start_time = Instant::now();
    event_loop.run(move |event, _, control_flow| {
        // Pass the winit events to the platform integration
        platform.handle_event(&event);

        match event {
            RedrawRequested(..) => {
                platform.update_time(start_time.elapsed().as_secs_f64());

                let output_frame = match surface.get_current_texture() {
                    Ok(frame) => frame,
                    Err(e) => {
                        eprintln!("Dropped frame with error: {}", e);
                        return;
                    }
                };

                let output_view = output_frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                // Begin to draw the UI frame
                platform.begin_frame();

                // Draw the primary panel
                CentralPanel::default()
                    .frame(
                        egui::Frame::central_panel(&platform.context().style())
                            .inner_margin(0.0)
                            .fill(style.tab_color),
                    )
                    .show(&platform.context(), |ui| {
                        // alt-tab'ing between tabs
                        if ui.input_mut(|i| i.consume_key(egui::Modifiers::ALT, egui::Key::Tab)) {
                            let focused_idx = match tree.focused_leaf() {
                                Some(idx) => idx,
                                None => egui_dock::NodeIndex::root(),
                            };

                            // don't do tab'ing if there are no tabs
                            if tree.len() == 0 {
                                return;
                            }

                            let focused = &mut tree[focused_idx];
                            if let egui_dock::Node::Leaf { tabs, active, .. } = focused {
                                if active.0 != tabs.len() - 1 {
                                    let tab_idx = active.0 + 1;
                                    tree.set_active_tab(focused_idx, egui_dock::TabIndex(tab_idx));
                                } else {
                                    tree.set_active_tab(focused_idx, egui_dock::TabIndex(0));
                                }
                            }
                        }

                        title_bar_ui(ui, &mut platform);

                        egui_dock::DockArea::new(&mut tree)
                            .style(dock_style.clone())
                            .show_close_buttons(buffers.has_multiple_tabs())
                            .draggable_tabs(buffers.has_multiple_tabs())
                            .show_inside(ui, &mut buffers);
                    });

                // end the UI frame. We could now handle the output and draw the UI with the backend
                let full_output = platform.end_frame(Some(&window));
                let paint_jobs = platform.context().tessellate(full_output.shapes);

                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("encoder"),
                });

                // upload all resources for the GPU
                let screen_descriptor = ScreenDescriptor {
                    physical_width: surface_config.width,
                    physical_height: surface_config.height,
                    scale_factor: window.scale_factor() as f32,
                };

                let tdelta: egui::TexturesDelta = full_output.textures_delta;

                egui_rpass
                    .add_textures(&device, &queue, &tdelta)
                    .expect("add texture ok");

                egui_rpass.update_buffers(&device, &queue, &paint_jobs, &screen_descriptor);

                // Record all render passes.
                egui_rpass
                    .execute(
                        &mut encoder,
                        &output_view,
                        &paint_jobs,
                        &screen_descriptor,
                        Some(wgpu::Color::BLACK),
                    )
                    .unwrap();
                // Submit the commands.
                queue.submit(std::iter::once(encoder.finish()));

                // Redraw egui
                output_frame.present();

                egui_rpass
                    .remove_textures(tdelta)
                    .expect("remove texture ok");
            }
            MainEventsCleared | UserEvent(Event::RequestRedraw) => {
                window.request_redraw();
            }
            WindowEvent { event, .. } => match event {
                winit::event::WindowEvent::Resized(size) => {
                    if size.width > 0 && size.height > 0 {
                        surface_config.width = size.width;
                        surface_config.height = size.height;
                        surface.configure(&device, &surface_config);
                    }
                }
                winit::event::WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }
                _ => {}
            },
            _ => (),
        }
    });
}

fn title_bar_ui(ui: &mut egui::Ui, platform: &mut Platform) {
    egui::menu::bar(ui, |ui| {
        ui.menu_button("File", |ui| {
            if ui.button(icon!(FOLDER_OPEN, "open")).clicked() {
                ui.close_menu();
            }
        });

        ui.menu_button("Edit", |ui| {
            ui.menu_button("My sub-menu", |ui| {
                if ui.button("Close the menu").clicked() {
                    ui.close_menu();
                }
            });
        });

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Max), |ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            close_maximize_minimize(ui, platform);
        });
    });
}

// Show some close/maximize/minimize buttons for the native window.
fn close_maximize_minimize(ui: &mut egui::Ui, platform: &mut Platform) {
    let height = 12.0;
    let close_response = ui.add(Button::new(RichText::new(icon!(CROSS, "")).size(height)));

    if close_response.clicked() {
        // platform.close();
    }

    // if platform.window_info.maximized {
    //     let maximized_response = ui
    //         .add(Button::new(RichText::new("🗗").size(button_height)));

    //     if maximized_response.clicked() {
    //         // platform.set_maximized(false);
    //     }
    // } else {
    let maximized_response = ui.add(Button::new(
        RichText::new(icon!(CHECKBOX_UNCHECKED, "")).size(height),
    ));

    if maximized_response.clicked() {
        // platform.set_maximized(true);
    }

    let minimized_response = ui.add(Button::new(RichText::new(icon!(MINUS, "")).size(height)));

    if minimized_response.clicked() {
        // platform.set_minimized(true);
    }
}
