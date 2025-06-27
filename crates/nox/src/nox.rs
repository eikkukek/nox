mod init_settings;

use winit::{
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    application::ApplicationHandler,
    event::WindowEvent,
    window::{WindowId, Window},
    dpi::LogicalSize,
};

use super::{
    interface::Interface,
    memory::Memory,
    renderer::Renderer,
    string_types::{ArrayString, LargeError},
};

pub use init_settings::InitSettings;


pub type AppName = ArrayString<128>;

pub type ShaderID = u64;

pub struct Nox<'interface, 'mem, I>
    where
        'mem: 'interface,
        I: Interface,
{
    interface: Option<&'interface mut I>,
    window: Option<Window>,
    memory: &'mem Memory,
    renderer: Option<Renderer<'mem>>,
}

impl<'interface, 'mem, I: Interface> Nox<'interface, 'mem, I>
    where
        'mem: 'interface
{

    pub fn new(interface: &'interface mut I, memory: &'mem mut Memory) -> Option<Self> {
        Some(
            Nox {
                interface: Some(interface),
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

    pub fn get_renderer(&mut self) -> Option<&mut Renderer<'mem>> {
        Some(self.renderer.as_mut()?)
    }

    pub fn load_shader(_input_filename: &str) -> Result<ShaderID, LargeError> {
        Err(LargeError::new())
    }
}

impl<'interface, 'mem, I: Interface> Drop for Nox<'interface, 'mem, I> {

    fn drop(&mut self) {
        if let Some(mut renderer) = self.renderer.take() {
            renderer.clean_up(self.memory.renderer_allocators());
        }
    }
}

impl<'interface, 'mem, I: Interface> ApplicationHandler for Nox<'interface, 'mem, I>
    where
        'mem: 'interface
{

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
                let renderer_allocators = self.memory.renderer_allocators();
                if let Some(window) = &self.window {
                    window.request_redraw();
                    if let Some(renderer) = &mut self.renderer {
                        if let Err(e) = renderer.render(&window, interface, renderer_allocators) {
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
                .with_inner_size(init_settings.window_size)
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
                    true,
                    *self.memory.renderer_layout(),
                    3,
                    renderer_allocators,
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
