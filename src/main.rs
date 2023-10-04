use std::num::NonZeroU32;

use eyre::{format_err, Result, WrapErr};
use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

struct State {
    // reverse order is important for safety (though I doubt anyone cares at exit)
    surface: softbuffer::Surface,
    _context: softbuffer::Context,
    window: Window,
}

impl State {
    pub fn new(event_loop: &EventLoop<()>) -> Result<Self> {
        let window = Window::new(event_loop).wrap_err("while creating window")?;

        // SAFETY: window is only dropped at the end of the struct
        // can't use wrap_err since softbuffer's error is neither send nor sync
        let context = unsafe { softbuffer::Context::new(&window) }
            .map_err(|e| format_err!("while creating softbuffer context: {e}"))?;
        let surface = unsafe { softbuffer::Surface::new(&context, &window) }
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

    pub fn process(&mut self, event: Event<()>) -> Result<ControlFlow> {
        match event {
            Event::RedrawRequested(_) => self.draw()?,
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(new_size) => self.resize(new_size)?,
                WindowEvent::CloseRequested => return Ok(ControlFlow::Exit),
                _ => (),
            },
            _ => (),
        }

        Ok(ControlFlow::Wait)
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
    let mut state = State::new(&event_loop).wrap_err("while creating state")?;

    event_loop
        .run(move |event, _, flow| {
            *flow = match state.process(event) {
                Ok(flow) => flow,
                Err(e) => {
                    eprintln!("error: {e}");
                    ControlFlow::Exit
                }
            }
        })
        .wrap_err("while running event loop")
}
