use std::{num::NonZeroU32, time::{Duration, Instant}};

use clap::Parser;
use eyre::{format_err, Result, WrapErr};
use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget},
    window::Window,
};

#[derive(Parser)]
struct Args {
    #[clap(long = "strobe", default_value_t = false)]
    strobe: bool,
}

#[derive(Default)]
struct Strobe {
    on: bool,
}

impl Strobe {
    fn toggle(&mut self) {
        self.on = !self.on;
    }
}

struct State<'win> {
    surface: softbuffer::Surface<&'win Window, &'win Window>,
    _context: softbuffer::Context<&'win Window>,
    window: &'win Window,

    strobe: Option<Strobe>,
}

impl<'win> State<'win> {
    pub fn new(window: &'win Window, args: Args) -> Result<Self> {
        let context = softbuffer::Context::new(window)
            .map_err(|e| format_err!("while creating softbuffer context: {e}"))?;
        let surface = softbuffer::Surface::new(&context, window)
            .map_err(|e| format_err!("while creating surface: {e}"))?;

        let mut state = Self {
            surface,
            _context: context,
            window,
            strobe: args.strobe.then(Strobe::default)
        };

        // initial resize needed for the surface to configure itself
        let initial_size = state.window.inner_size();
        state.resize(initial_size)?;

        Ok(state)
    }

    pub fn process(&mut self, elwt: &EventLoopWindowTarget<()>, event: Event<()>) -> Result<()> {
        let flow = if self.strobe.is_some() { 
            self.draw()?;

            let now = Instant::now();
            let fps = 20;
            let next_frame = now + Duration::from_millis(1000 / fps);

            ControlFlow::WaitUntil(next_frame)
        } else {
            ControlFlow::Wait
        };

        dbg!(flow);

        elwt.set_control_flow(flow);
        if let Event::WindowEvent { event, .. } = event {
            match event {
                WindowEvent::RedrawRequested if self.strobe.is_none() => self.draw()?,
                WindowEvent::Resized(new_size) => self.resize(new_size)?,
                WindowEvent::CloseRequested => elwt.exit(),
                _ => (),
            }
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

        let off_color = 0;
        let on_color = (255 << 16) | (255 << 8) | 255;

        buffer.fill(on_color);

        if let Some(ref mut strobe) = self.strobe.as_mut() {
            if !dbg!(strobe.on) {
                buffer.fill(off_color);
            }
            strobe.toggle();
        }

        buffer
            .present()
            .map_err(|e| format_err!("could not present display buffer: {e}"))
    }
}

fn main() -> Result<()> {
    let args = Args::parse();
    let event_loop = EventLoop::new().wrap_err("while creating event loop")?;
    let window = Window::new(&event_loop).wrap_err("while creating window")?;
    let mut state = State::new(&window, args).wrap_err("while creating state")?;

    event_loop
        .run(move |event, elwt| {
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
