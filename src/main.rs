use std::num::NonZeroU32;

use eyre::{format_err, Result, WrapErr};
use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget},
    window::Window,
};

struct State<'win> {
    surface: softbuffer::Surface<&'win Window, &'win Window>,
    _context: softbuffer::Context<&'win Window>,
    window: &'win Window,
}

impl<'win> State<'win> {
    pub fn new(window: &'win Window) -> Result<Self> {
        let context = softbuffer::Context::new(window)
            .map_err(|e| format_err!("while creating softbuffer context: {e}"))?;
        let surface = softbuffer::Surface::new(&context, window)
            .map_err(|e| format_err!("while creating surface: {e}"))?;

        let mut state = Self {
            surface,
            _context: context,
            window,
        };

        // initial resize needed for the surface to configure itself
        let initial_size = state.window.inner_size();
        state.resize(initial_size)?;

        Ok(state)
    }

    pub fn process(&mut self, elwt: &EventLoopWindowTarget<()>, event: Event<()>) -> Result<()> {
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::RedrawRequested => self.draw()?,
                WindowEvent::Resized(new_size) => self.resize(new_size)?,
                WindowEvent::CloseRequested => elwt.exit(),
                _ => (),
            },
            _ => (),
        }

        Ok(())
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) -> Result<()> {
        let Some(width) = NonZeroU32::new(new_size.width) else {
            // probably minimized, won't be drawn then anyway
            return Ok(());
        };
        let Some(height) = NonZeroU32::new(new_size.height) else {
            return Ok(());
        };

        self.surface
            .resize(width, height)
            .map_err(|e| format_err!("could not resize surface: {e}"))
    }

    pub fn draw(&mut self) -> Result<()> {
        let mut buffer = self
            .surface
            .buffer_mut()
            .map_err(|e| format_err!("could not get reference to display buffer: {e}"))?;

        let light_color = (255 << 16) | (255 << 8) | 255;
        buffer.fill(light_color);

        buffer
            .present()
            .map_err(|e| format_err!("could not present display buffer: {e}"))
    }
}

fn main() -> Result<()> {
    let event_loop = EventLoop::new().wrap_err("while creating event loop")?;
    let window = Window::new(&event_loop).wrap_err("while creating window")?;
    let mut state = State::new(&window).wrap_err("while creating state")?;

    event_loop
        .run(move |event, elwt| {
            elwt.set_control_flow(ControlFlow::Wait);
            match state.process(elwt, event) {
                Ok(flow) => flow,
                Err(e) => {
                    eprintln!("error: {e}");
                    elwt.exit();
                }
            }
        })
        .wrap_err("while running event loop")
}
