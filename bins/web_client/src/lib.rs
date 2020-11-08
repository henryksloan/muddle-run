use wasm_bindgen::prelude::*;
use winit::event_loop::EventLoopProxy;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

struct WindowSetup {
    window: winit::window::Window,
    event_loop: EventLoop<WgpuSetup>,
    event_loop_proxy: EventLoopProxy<WgpuSetup>,
    size: winit::dpi::PhysicalSize<u32>,
}

struct WgpuSetup {
    window: winit::window::Window,
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
}

struct Pass {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    uniform_buf: wgpu::Buffer,
}

pub struct Renderer;

impl Renderer {
    fn optional_features() -> wgpu::Features {
        wgpu::Features::empty()
    }

    fn required_features() -> wgpu::Features {
        wgpu::Features::empty()
    }

    fn required_limits() -> wgpu::Limits {
        wgpu::Limits::default()
    }

    fn new(
        sc_desc: &wgpu::SwapChainDescriptor,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self {
        Self
    }

    fn resize(
        &mut self,
        sc_desc: &wgpu::SwapChainDescriptor,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    );
    fn update(&mut self, event: WindowEvent);
    fn render(
        &mut self,
        frame: &wgpu::SwapChainTexture,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        spawner: &impl LocalSpawn,
    );
}

#[wasm_bindgen(start)]
pub fn main() {
    let title = "muddle.run".to_owned();
    let WindowSetup {
        window,
        event_loop,
        event_loop_proxy,
        size,
        ..
    } = window_setup(&title);
    wasm_bindgen_futures::spawn_local(async move {
        wgpu_setup::<E>(window, event_loop_proxy).await;
    });
    start::<E>(event_loop, size);
}

fn window_setup(title: &str) -> WindowSetup {
    let event_loop = EventLoop::<WgpuSetup>::with_user_event();
    let event_loop_proxy = event_loop.create_proxy();
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    let mut builder = winit::window::WindowBuilder::new();
    builder = builder.with_title(title);
    let window = builder.build(&event_loop).unwrap();
    let size = window.inner_size();

    {
        use winit::platform::web::WindowExtWebSys;
        // On wasm, append the canvas to the document body
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| doc.body())
            .and_then(|body| {
                body.append_child(&web_sys::Element::from(window.canvas()))
                    .ok()
            })
            .expect("couldn't append canvas to document body");
    }

    WindowSetup {
        event_loop,
        event_loop_proxy,
        window,
        size,
    }
}

async fn wgpu_setup<E: Example>(
    window: winit::window::Window,
    event_loop_proxy: winit::event_loop::EventLoopProxy<WgpuSetup>,
) {
    log::info!("Initializing the surface...");

    let instance = wgpu::Instance::new();
    let surface = unsafe { instance.create_surface(&window) };

    let adapter = instance
        .request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::Default,
                compatible_surface: Some(&surface),
            },
            wgpu::BackendBit::PRIMARY,
        )
        .await
        .unwrap();
    let optional_features = E::optional_features();
    let required_features = E::required_features();
    let adapter_features = adapter.features();
    assert!(
        adapter_features.contains(required_features),
        "Adapter does not support required features for this example: {:?}",
        required_features - adapter_features
    );
    let needed_limits = E::required_limits();
    let trace_dir = std::env::var("WGPU_TRACE");
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                extensions: wgpu::Extensions {
                    anisotropic_filtering: false,
                },
                limits: wgpu::Limits::default(),
            },
            trace_dir.ok().as_ref().map(std::path::Path::new),
        )
        .await
        .unwrap();

    let send_event_result = event_loop_proxy.send_event(WgpuSetup {
        window,
        surface,
        device,
        queue,
    });

    match send_event_result {
        Ok(_) => {}
        Err(_) => {
            log::warn!("Could not send event that wgpu has been initialized. Was the event loop destroyed right away?");
        }
    }
}

struct Application<E> {
    wgpu_setup: WgpuSetup,
    example: E,
    swap_chain: wgpu::SwapChain,
}

fn start<E: Example>(
    event_loop: EventLoop<()>,
    window: Window,
    swapchain_format: wgpu::TextureFormat,
) {
    let size = window.inner_size();
    let instance = wgpu::Instance::new();
    let surface = unsafe { instance.create_surface(&window) };
    let adapter = instance
        .request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::Default,
                compatible_surface: Some(&surface),
            },
            wgpu::BackendBit::PRIMARY,
        )
        .await
        .unwrap();

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                extensions: wgpu::Extensions {
                    anisotropic_filtering: false,
                },
                limits: wgpu::Limits::default(),
            },
            None,
        )
        .await
        .unwrap();

    let vs = include_bytes!("../../../resources/shaders/shader.vert.spv");
    let vs_module =
        device.create_shader_module(&wgpu::read_spirv(std::io::Cursor::new(&vs[..])).unwrap());

    let fs = include_bytes!("../../../resources/shaders/shader.frag.spv");
    let fs_module =
        device.create_shader_module(&wgpu::read_spirv(std::io::Cursor::new(&fs[..])).unwrap());

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        bind_group_layouts: &[],
    });

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        layout: &pipeline_layout,
        vertex_stage: wgpu::ProgrammableStageDescriptor {
            module: &vs_module,
            entry_point: "main",
        },
        fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
            module: &fs_module,
            entry_point: "main",
        }),
        rasterization_state: Some(wgpu::RasterizationStateDescriptor {
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: wgpu::CullMode::None,
            depth_bias: 0,
            depth_bias_slope_scale: 0.0,
            depth_bias_clamp: 0.0,
        }),
        primitive_topology: wgpu::PrimitiveTopology::TriangleList,
        color_states: &[wgpu::ColorStateDescriptor {
            format: swapchain_format,
            color_blend: wgpu::BlendDescriptor::REPLACE,
            alpha_blend: wgpu::BlendDescriptor::REPLACE,
            write_mask: wgpu::ColorWrite::ALL,
        }],
        depth_stencil_state: None,
        vertex_state: wgpu::VertexStateDescriptor {
            index_format: wgpu::IndexFormat::Uint16,
            vertex_buffers: &[],
        },
        sample_count: 1,
        sample_mask: !0,
        alpha_to_coverage_enabled: false,
    });

    let mut sc_desc = wgpu::SwapChainDescriptor {
        usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
        format: swapchain_format,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Mailbox,
    };

    let mut swap_chain = device.create_swap_chain(&surface, &sc_desc);

    event_loop.run(move |event, _, control_flow| {
        // log::info!("Hello, world!");

        // force ownership by the closure
        let _ = (
            &instance,
            &adapter,
            &vs_module,
            &fs_module,
            &pipeline_layout,
        );

        *control_flow = ControlFlow::Poll;
        match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                sc_desc.width = size.width;
                sc_desc.height = size.height;
                swap_chain = device.create_swap_chain(&surface, &sc_desc);
            }
            Event::RedrawRequested(_) => {
                let frame = swap_chain
                    .get_next_frame()
                    .expect("Failed to acquire next swap chain texture")
                    .output;
                let mut encoder =
                    device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
                {
                    let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                            attachment: &frame.view,
                            resolve_target: None,
                            load_op: wgpu::LoadOp::Clear,
                            store_op: wgpu::StoreOp::Store,
                            clear_color: wgpu::Color::GREEN,
                        }],
                        depth_stencil_attachment: None,
                    });
                    rpass.set_pipeline(&render_pipeline);
                    rpass.draw(0..3, 0..1);
                }

                queue.submit(Some(encoder.finish()));
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => {}
        }
    });
}
