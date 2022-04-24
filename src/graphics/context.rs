use vulkano::{
    command_buffer::{
        AutoCommandBufferBuilder, CommandBufferUsage, PrimaryAutoCommandBuffer,
        PrimaryCommandBuffer, SubpassContents,
    },
    device::physical::{PhysicalDevice, PhysicalDeviceType},
    device::{Device, DeviceCreateInfo, DeviceExtensions, QueueCreateInfo},
    image::{view::ImageView, ImageUsage, SwapchainImage},
    instance::{Instance, InstanceCreateInfo},
    pipeline::{graphics::vertex_input::BuffersDefinition, graphics::viewport::Viewport},
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass},
    sampler::{Filter, Sampler, SamplerAddressMode, SamplerCreateInfo},
    swapchain::{self, AcquireError, Swapchain, SwapchainCreationError, SwapchainCreateInfo},
    sync::{self, FlushError, GpuFuture},
    Version,
};

use vulkano_win::VkSurfaceBuild;
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowBuilder};

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use vulkano::image::ImageAccess;

use crate::{conf::*, graphics::shader::ShaderId, graphics::*};

/// This is the context from which the graphics components gets all of its information
/// about the physical device and the presentation area. It serves as the Vulkano abstraction,
/// which intern interfaces with the Vulkan API.
///
/// # Examples
///
/// ```
/// use winit::{
///     event_loop::{ControlFlow},
///     event::{Event, WindowEvent}
/// };
/// use ledge_engine::graphics::context::GraphicsContext;
/// use ledge_engine::conf::Conf;
///
/// fn main() {
///     let (mut context, event_loop) = GraphicsContext::new(Conf::default());
///
///     event_loop.run(move |event, _, control_flow| {
///         let now = std::time::Instant::now();
///
///         match event {
///             Event::WindowEvent { event, .. } => match event {
///                 WindowEvent::CloseRequested => {
///                     *control_flow = ControlFlow::Exit;
///                 },
///                 WindowEvent::Resized(_) => {
///                     context.recreate_swapchain = true;
///                 },
///                 _ => {},
///             },
///             Event::MainEventsCleared => {
///                 context.create_command_buffer();
///
///                 // buffer updates
///
///                 context.begin_frame();
///
///                 // draw commands
///
///                 context.present();
///
///                 // without using timer you have to manually control the frame time.
///                 let mut sleep_time: f64 = 0.016 - now.elapsed().as_secs_f64();
///                 if sleep_time < 0.0 {
///                     sleep_time = 0.0
///                 }

///                 std::thread::sleep(std::time::Duration::from_secs_f64(sleep_time));
///                 print!("{:.2}\r", now.elapsed().as_secs_f32() * 1000.0);
///             },
///             _ => {}
///         }
///     });
/// }
/// ```
#[allow(unused)]
pub struct GraphicsContext {
    pub(crate) queue: Arc<vulkano::device::Queue>,
    pub(crate) surface: Arc<vulkano::swapchain::Surface<winit::window::Window>>,
    pub device: Arc<vulkano::device::Device>,
    pub(crate) swapchain: Arc<vulkano::swapchain::Swapchain<winit::window::Window>>,
    pub(crate) framebuffers: std::vec::Vec<Arc<vulkano::render_pass::Framebuffer>>,
    pub(crate) render_pass: Arc<RenderPass>,
    pub(crate) viewport: Viewport,
    pub(crate) image_num: usize,
    pub(crate) recreate_swapchain: bool,
    pub(crate) previous_frame_end: Option<Box<dyn vulkano::sync::GpuFuture>>,
    pub(crate) present_future: Option<Box<dyn vulkano::sync::GpuFuture>>,
    pub command_buffer: Option<AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>>,
    pub default_shader: ShaderId,
    pub(crate) current_shader: Rc<RefCell<Option<ShaderId>>>,
    pub shaders: Vec<Arc<dyn crate::graphics::shader::ShaderHandle>>,
    pub(crate) samplers: Vec<Arc<Sampler>>,
}

impl GraphicsContext {
    pub fn new(_conf: Conf) -> (Self, winit::event_loop::EventLoop<()>) {
        let required_extensions = vulkano_win::required_extensions();
        let instance = Instance::new(InstanceCreateInfo {
            application_name: None, 
            application_version: Version::V1_1, 
            enabled_extensions: required_extensions, 
            ..Default::default()
        }).unwrap();

        let event_loop = EventLoop::new();
        let surface = WindowBuilder::new()
            .build_vk_surface(&event_loop, instance.clone())
            .unwrap();

        let device_extensions = DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::none()
        };
        let (physical_device, queue_family) = PhysicalDevice::enumerate(&instance)
            .filter(|&p| p.supported_extensions().is_superset_of(&device_extensions))
            .filter_map(|p| {
                p.queue_families()
                    .find(|&q| q.supports_graphics() && q.supports_surface(&surface).unwrap_or(false))
                    .map(|q| (p, q))
            })
            .min_by_key(|(p, _)| match p.properties().device_type {
                PhysicalDeviceType::DiscreteGpu => 0,
                PhysicalDeviceType::IntegratedGpu => 1,
                PhysicalDeviceType::VirtualGpu => 2,
                PhysicalDeviceType::Cpu => 3,
                PhysicalDeviceType::Other => 4,
            })
            .unwrap();

        println!(
            "Using device: {} (type: {:?})",
            physical_device.properties().device_name,
            physical_device.properties().device_type,
        );

        let (device, mut queues) = Device::new(
            physical_device,
            DeviceCreateInfo {
                enabled_extensions: physical_device
                    .required_extensions()
                    .union(&device_extensions),
                queue_create_infos: vec![QueueCreateInfo::family(queue_family)],
                ..Default::default()
            },
        )
        .unwrap();

        let queue = queues.next().unwrap();

        let (swapchain, images) = {
            let surface_capabilities = physical_device
                .surface_capabilities(&surface, Default::default())
                .unwrap();
    
            let image_format = Some(
                physical_device
                    .surface_formats(&surface, Default::default())
                    .unwrap()[0]
                    .0,
            );
    
            Swapchain::new(
                device.clone(),
                surface.clone(),
                SwapchainCreateInfo {
                    min_image_count: surface_capabilities.min_image_count,
                    image_format,
                    image_extent: surface.window().inner_size().into(),
                    image_usage: ImageUsage::color_attachment(),
                    composite_alpha: surface_capabilities
                        .supported_composite_alpha
                        .iter()
                        .next()
                        .unwrap(),
                    ..Default::default()
                },
            )
            .unwrap()
        };

        let render_pass = vulkano::single_pass_renderpass!(device.clone(),
            attachments: {
                color: {
                    load: Clear,
                    store: Store,
                    format: swapchain.image_format(),
                    samples: 1,
                }
            },
            pass: {
                color: [color],
                depth_stencil: {}
            }
        )
        .unwrap();

        let default_future = sync::now(device.clone()).boxed();

        let mut viewport = Viewport {
            origin: [0.0, 0.0],
            dimensions: [0.0, 0.0],
            depth_range: 0.0..1.0,
        };

        let framebuffers = window_size_dependent_setup(
            &images, 
            render_pass.clone(), 
            &mut viewport,
        );

        let mut samplers = Vec::new();

        let default_sampler = Sampler::new(
            device.clone(),
            SamplerCreateInfo {
                mag_filter: Filter::Linear,
                min_filter: Filter::Linear,
                address_mode: [SamplerAddressMode::Repeat; 3],
                ..Default::default()
            },
        )
        .unwrap();

        samplers.push(default_sampler);

        let mut context = GraphicsContext {
            queue,
            surface,
            device,
            swapchain,
            framebuffers,
            render_pass,
            viewport,
            image_num: 0,
            present_future: None,
            previous_frame_end: Some(default_future),
            recreate_swapchain: false,
            command_buffer: None,
            default_shader: 0,
            current_shader: Rc::new(RefCell::new(None)),
            shaders: Vec::new(),
            samplers,
        };

        let v_shader = vs::load(context.device.clone()).unwrap();
        let f_shader = fs::load(context.device.clone()).unwrap();

        let default_program = shader::ShaderProgram::new(
            &mut context,
            BuffersDefinition::new()
                .vertex::<Vertex>()
                .instance::<InstanceData>(),
            shader::VertexTopology::TriangleStrip,
            v_shader.entry_point("main").unwrap(),
            f_shader.entry_point("main").unwrap(),
            BlendMode::Alpha,
        );

        context.shaders.push(Arc::new(default_program));

        (context, event_loop)
    }

    pub fn recreate_swapchain(&mut self) {
        self.recreate_swapchain = true
    }

    fn create_command_buffer(&mut self) {
        self.command_buffer = Some(
            AutoCommandBufferBuilder::primary(
                self.device.clone(),
                self.queue.family(),
                CommandBufferUsage::OneTimeSubmit,
            )
            .unwrap(),
        );
    }

    /// Handles setup of a new frame, called when the graphics pipeline is first created and
    /// at the end of every frame to start the next one.
    ///
    /// This is necessary because the swapchain could be out of date,
    /// as well as updating the image_num, optimality, and the swapcahin future.
    pub fn begin_frame(&mut self, color: Color) {
        self.create_command_buffer();

        self.previous_frame_end.as_mut().unwrap().cleanup_finished();

        if self.recreate_swapchain {
            let (new_swapchain, new_images) =
                match self.swapchain.recreate(SwapchainCreateInfo {
                    image_extent: self.surface.window().inner_size().into(),
                    ..self.swapchain.create_info()
                }) {
                    Ok(r) => r,
                    Err(SwapchainCreationError::ImageExtentNotSupported { .. }) => return,
                    Err(e) => panic!("Failed to recreate swapchain: {:?}", e),
                };

            self.swapchain = new_swapchain;
            self.framebuffers = window_size_dependent_setup(
                &new_images,
                self.render_pass.clone(),
                &mut self.viewport,
            );
            self.recreate_swapchain = false;
        }

        let (image_num, suboptimal, acquire_future) =
            match swapchain::acquire_next_image(self.swapchain.clone(), None) {
                Ok(r) => r,
                Err(AcquireError::OutOfDate) => {
                    self.recreate_swapchain = true;
                    return;
                }
                Err(e) => panic!("Failed to acquire next image: {:?}", e),
            };

        self.present_future = Some(
            self.previous_frame_end
                .take()
                .unwrap()
                .join(acquire_future)
                .boxed()
        );

        if suboptimal {
            self.recreate_swapchain = true;
        }

        self.image_num = image_num;

        let color_value: [f32; 4] = color.into();
        let clear_values = vec![color_value.into()];

        self.command_buffer
            .as_mut()
            .unwrap()
            .begin_render_pass(
                self.framebuffers[self.image_num].clone(),
                SubpassContents::Inline,
                clear_values,
            )
            .unwrap();

        self.command_buffer.as_mut().unwrap().set_viewport(
            0,
            vec![self.viewport.clone()],
        );

        let shader_handle = self.shaders[0].clone();
        self.command_buffer
            .as_mut()
            .unwrap()
            .bind_pipeline_graphics(shader_handle.pipeline().clone());
    }

    /// Interacts with the given shader handle (which by default is a ```ledge_engine::graphics::shader::ShaderProgram```)
    /// to use that specific shader to draw the vertex buffer to the screen.
    pub fn draw(&mut self, pipe_data: Box<dyn PipelineData>) {
        let id = (*self.current_shader.borrow()).unwrap_or(self.default_shader);
        let shader_handle = &self.shaders[id];

        shader_handle.draw(&mut self.command_buffer.as_mut().unwrap(), pipe_data);
    }

    /// This function submits the command buffer to the queue and fences the operation,
    /// storing a future refering to the operation.
    ///
    /// This function must be run once at the end of all updates and draw calls in order for the frame to be sumbitted.
    pub fn present(&mut self) {
        self.command_buffer
            .as_mut()
            .unwrap()
            .end_render_pass()
            .unwrap();

        let command_buffer = self.command_buffer.take().unwrap().build().unwrap();

        let future = command_buffer
            .execute_after(self.present_future.take().unwrap(), self.queue.clone())
            .unwrap();

        let future = swapchain::present(
            self.swapchain.clone(),
            future,
            self.queue.clone(),
            self.image_num,
        );

        let future = future.then_signal_fence_and_flush();

        match future {
            Ok(future) => {
                self.previous_frame_end = Some(future.boxed());
            }
            Err(FlushError::OutOfDate) => {
                self.recreate_swapchain = true;
                self.previous_frame_end = Some(sync::now(self.device.clone()).boxed());
            }
            Err(e) => {
                println!("Failed to flush future: {:?}", e);
                self.previous_frame_end = Some(sync::now(self.device.clone()).boxed());
            }
        };
    }

    pub fn set_blend_mode(&mut self, _mode: BlendMode) {}
}

fn window_size_dependent_setup(
    images: &[Arc<SwapchainImage<Window>>],
    render_pass: Arc<RenderPass>,
    viewport: &mut Viewport,
) -> Vec<Arc<Framebuffer>> {
    let dimensions = images[0].dimensions().width_height();
    viewport.dimensions = [dimensions[0] as f32, dimensions[1] as f32];

    images
        .iter()
        .map(|image| {
            let view = ImageView::new_default(image.clone()).unwrap();
            Framebuffer::new(
                render_pass.clone(),
                FramebufferCreateInfo {
                    attachments: vec![view],
                    ..Default::default()
                },
            )
            .unwrap()
        })
        .collect::<Vec<_>>()
}
