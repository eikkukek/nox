use super::{
    string::String,
    version::Version,
    backend::{self, Backend, DeviceName},
};

use winit::{
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    application::ApplicationHandler,
    event::WindowEvent,
    window::{WindowId, Window},
    dpi::LogicalSize,
};

const MAX_APP_NAME_LENGTH: usize = 64;

pub type AppName = String<MAX_APP_NAME_LENGTH>;

pub struct InitSettings {
    pub app_name: AppName,
    pub app_version: Version,
    pub window_width: f32,
    pub window_height: f32,
}

impl InitSettings {

    pub fn new(
        app_name: &str,
        app_version: Version,
        window_width: f32,
        window_height: f32
    ) -> Self
    {
        InitSettings {
            app_name: String::from_str(app_name),
            app_version,
            window_width,
            window_height,
        }
    }
}

pub struct Memory {
    backend_mem: backend::Memory,
}

impl Memory {

    pub fn default() -> Option<Self> {
        Some(
            Self {
                backend_mem: backend::Memory::default()?,
            }
        )
    }
}

pub struct Nox<'mem> {
    init_settings: InitSettings,
    window: Option<Window>,
    backend: Backend<'mem>,
}

impl<'mem> Nox<'mem> {

    pub fn new(init_settings: InitSettings, memory: &'mem mut Memory) -> Option<Self> {
        let backend = Backend::new(
            backend::InitSettings::new(
                init_settings.app_name,
                init_settings.app_version.as_u32(),
                true,
            ),
            &mut memory.backend_mem
        )?;
        Some(
            Nox {
                init_settings,
                window: None,
                backend,
            }
        )
    }

    pub fn run(mut self) {
        let event_loop = EventLoop::new().unwrap();
        event_loop.set_control_flow(ControlFlow::Poll);
        event_loop.run_app(&mut self).expect("failed to run event loop");
    }

    pub fn get_gpu_name(&self) -> Option<DeviceName> {
        self.backend.get_physical_device_name()
    }
}

impl<'mem> Drop for Nox<'mem> {

    fn drop(&mut self) {
    }
}

impl<'mem> ApplicationHandler for Nox<'mem> {

    fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(), // terminate app,
            WindowEvent::RedrawRequested => {
                if let Some(window) = &self.window {
                    assert!(window_id == window.id());
                    window.request_redraw();
                }
            },
            _ => {},
        }
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window_attributes = Window::default_attributes()
                .with_title(self.init_settings.app_name.as_str())
                .with_inner_size(LogicalSize::new(self.init_settings.window_width, self.init_settings.window_height))
                .with_min_inner_size(LogicalSize::new(1.0, 1.0));
            let window = match event_loop.create_window(window_attributes) {
                Ok(window) => window,
                Err(e) => {
                    eprintln!("Nox error: failed to create window ( {} )", e);
                    event_loop.exit();
                    return
                },
            };
            println!("Nox message: created window {}", self.init_settings.app_name);
            event_loop.set_control_flow(ControlFlow::Poll);
            window.request_redraw();
            if let Err(e) = self.backend.init_vulkan(&window) {
                eprintln!("Nox error: failed to initialize backend ( {} )", e);
                event_loop.exit();
            }
            self.window = Some(window);
        }
    }
}
