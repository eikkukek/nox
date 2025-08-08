use std::sync::{Arc, RwLock};

use winit::{
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    application::ApplicationHandler,
    event::WindowEvent,
    window::{WindowId, Window},
    dpi::LogicalSize,
};

use nox_mem::string_types::ArrayString;

pub use crate::renderer;

use super::{
    interface::Interface,
    memory::Memory,
    renderer::Renderer,
};

pub type AppName = ArrayString<128>;

pub type ShaderID = u64;

pub struct Nox<'mem, I>
    where
        I: Interface,
{
    interface: Arc<RwLock<I>>,
    window: Option<Window>,
    memory: &'mem Memory,
    renderer: Option<Renderer<'mem>>,
}

impl<'mem, I: Interface> Nox<'mem, I>
{

    pub fn new(interface: I, memory: &'mem mut Memory) -> Option<Self> {
        Some(
            Nox {
                interface: Arc::new(RwLock::new(interface)),
                window: None,
                memory,
                renderer: None,
            }
        )
    }

    pub fn run(mut self) {
        let event_loop = EventLoop::new().unwrap();
        event_loop.set_control_flow(ControlFlow::Poll);
        event_loop.run_app(&mut self).expect("failed to run event loop");
    }

    pub fn gpu_name(&mut self) -> renderer::DeviceName {
        self.renderer
            .as_ref()
            .unwrap()
            .device_info()
            .device_name().clone()
    }
}

impl<'mem, I: Interface> Drop for Nox<'mem, I> {

    fn drop(&mut self) {
        if let Some(mut renderer) = self.renderer.take() {
            renderer.clean_up(self.memory.renderer_allocators());
        }
    }
}

impl<'mem, I: Interface> ApplicationHandler for Nox<'mem, I> {

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(), // terminate app,
            WindowEvent::Resized(size) => {
                if let Some(renderer) = &mut self.renderer {
                    renderer.request_resize(size);
                }
            },
            WindowEvent::RedrawRequested => {
                let interface = self.interface.clone();
                let mut renderer_context = self.renderer.as_mut().unwrap().renderer_context();
                interface.write().unwrap().update(
                    self,
                    &mut renderer_context,
                );
                let renderer_allocators = self.memory.renderer_allocators();
                if let Some(window) = &self.window {
                    window.request_redraw();
                    if let Some(renderer) = &mut self.renderer {
                        let mut handles = renderer.command_requests(self.interface.clone(), renderer_context.command_requests);
                        for handle in &mut handles {
                            if let Err(e) = handle.take().unwrap().join() {
                                panic!("thread poisoned during command requests: {:?}", e)
                            }
                        }
                        if let Err(e) = renderer.render(&window, self.interface.clone(), renderer_allocators) {
                            eprintln!("Nox renderer error: {}", e);
                            event_loop.exit();
                        }
                    }
                }
            },
            _ => {},
        }
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let init_settings = self.interface.read().unwrap().init_settings();
        if self.window.is_none() {
            let window_attributes = Window::default_attributes()
                .with_title(init_settings.app_name.as_str())
                .with_inner_size(LogicalSize::new(
                    init_settings.window_size[0],
                    init_settings.window_size[1],
                ))
                .with_min_inner_size(LogicalSize::new(1.0, 1.0));
            let window = match event_loop.create_window(window_attributes) {
                Ok(window) => window,
                Err(e) => {
                    eprintln!("Nox error: failed to create window ( {} )", e);
                    event_loop.exit();
                    return
                },
            };
            println!("Nox message: created window {}", init_settings.app_name);
            event_loop.set_control_flow(ControlFlow::Poll);
            let renderer_allocators = self.memory.renderer_allocators();
            window.request_redraw();
            self.renderer = match Renderer
                ::new(
                    &window,
                    &init_settings.app_name,
                    init_settings.app_version,
                    init_settings.enable_vulkan_validation,
                    *self.memory.renderer_layout(),
                    3,
                    renderer_allocators,
                ) {
                Ok(r) => Some(r),
                Err(e) => {
                    event_loop.exit();
                    eprintln!("Nox error: failed to create renderer ( {} )", e);
                    return
                }
            };
            self.window = Some(window);
            let mut renderer_context = self.renderer.as_mut().unwrap().renderer_context();
            if let Err(e) = self.interface
                .clone()
                .write()
                .unwrap()
                .init_callback(self, &mut renderer_context) {
                event_loop.exit();
                eprintln!("Nox error: init callback error ( {:?} )", e);
            }
            self.renderer.as_mut().unwrap().command_requests(self.interface.clone(), renderer_context.command_requests);
        }
    }
}
