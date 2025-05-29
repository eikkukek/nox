use super::{
    interface::Interface,
    backend::{self, Backend},
    renderer::Renderer,
    string::String,
    version::Version,
};

use winit::{
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    application::ApplicationHandler,
    event::WindowEvent,
    window::{WindowId, Window},
    dpi::LogicalSize,
};

pub type AppName = String<64>;
pub type Memory<'mem> = backend::Memory<'mem>;

pub struct Extent<T> {
    pub width: T,
    pub height: T,
}

impl<T> Extent<T>
    where
        T: Copy,
{

    pub fn new(width: T, height: T) -> Self {
        Self {
            width,
            height
        }
    }

    pub fn width(&mut self, width: T) {
        self.width = width;
    }

    pub fn height(&mut self, height: T) {
        self.height = height;
    }
    
    pub fn as_logical_size(&self) -> LogicalSize<T> {
        LogicalSize::new(self.width, self.height)
    }
}

pub struct InitSettings {
    pub app_name: AppName,
    pub app_version: Version,
    pub window_size: Extent<f32>
}

impl InitSettings {

    pub fn new(
        app_name: &str,
        app_version: Version,
        window_size: Extent<f32>
    ) -> Self
    {
        InitSettings {
            app_name: String::from_str(app_name),
            app_version,
            window_size,
        }
    }
}

pub struct Nox<'interface, I>
    where
        I: Interface,
{
    interface: Option<&'interface mut I>,
    window: Option<Window>,
    renderer: Option<Renderer<'interface>>,
    backend: Backend<'interface>, // contains memory for renderer
}

impl<'interface, I: Interface> Nox<'interface, I> {

    pub fn new(interface: &'interface mut I) -> Option<Self> {
        let backend = Backend::new(
            I::create_memory(),
        )?;
        Some(
            Nox {
                interface: Some(interface),
                window: None,
                backend,
                renderer: None,
            }
        )
    }

    pub fn run(mut self) {
        let event_loop = EventLoop::new().unwrap();
        event_loop.set_control_flow(ControlFlow::Poll);
        event_loop.run_app(&mut self).expect("failed to run event loop");
    }

    pub fn get_renderer(&mut self) -> Option<&mut Renderer<'interface>> {
        Some(self.renderer.as_mut()?)
    }
}

impl<'interface, I: Interface> Drop for Nox<'interface, I> {

    fn drop(&mut self) {
        if let Some(mut renderer) = self.renderer.take() {
            renderer.destroy();
        }
    }
}

impl<'interface, I: Interface> ApplicationHandler for Nox<'interface, I> {

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(), // terminate app,
            WindowEvent::Resized(size) => {
                if let Some(renderer) = &mut self.renderer {
                    renderer.request_resize(size);
                }
            },
            WindowEvent::RedrawRequested => {
                let Some(interface) = self.interface.take() else { return };
                if let Some(window) = &self.window {
                    window.request_redraw();
                    if let Some(renderer) = &mut self.renderer {
                        if let Err(e) = renderer.render(&window, interface, self.backend.renderer_memory()) {
                            eprintln!("Nox renderer error: {}", e);
                            event_loop.exit();
                        }
                    }
                }
                self.interface = Some(interface);
            },
            _ => {},
        }
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let Some(interface) = self.interface.take() else { return };
        let init_settings = interface.init_settings();
        if self.window.is_none() {
            let window_attributes = Window::default_attributes()
                .with_title(init_settings.app_name.as_str())
                .with_inner_size(init_settings.window_size.as_logical_size())
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
            window.request_redraw();
            self.renderer = match Renderer
                ::new(
                    &window,
                    &init_settings.app_name,
                    init_settings.app_version,
                    true,
                    self.backend.renderer_memory(),
                ) {
                Ok(r) => Some(r),
                Err(e) => {
                    event_loop.exit();
                    eprintln!("Nox error: failed to create renderer ( {} )", e);
                    None
                }
            };
            self.window = Some(window);
            interface.init_callback(self);
            self.interface = Some(interface);
        }
    }
}
