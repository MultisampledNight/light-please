use std::num::NonZeroU32;

use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window, dpi::PhysicalSize,
};

struct State {
    // reverse order is important for safety (though I doubt anyone cares at exit)
    surface: softbuffer::Surface,
    context: softbuffer::Context,
    window: Window,
}

impl State {
    pub fn new(event_loop: &EventLoop<()>) -> Self {
        let window = Window::new(event_loop).unwrap();

        // SAFETY: window is only dropped at the end of the struct
        let context = unsafe { softbuffer::Context::new(&window) }.unwrap();
        let surface = unsafe { softbuffer::Surface::new(&context, &window) }.unwrap();

        let mut state = Self {
            surface,
            context,
            window,
        };

        // initial resize needed for the surface to configure itself
        let initial_size = state.window.inner_size();
        state.resize(initial_size);

        state
    }

    pub fn process(&mut self, event: Event<()>) -> ControlFlow {
        match event {
            Event::RedrawRequested(_) => self.draw(),
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(new_size) => self.resize(new_size),
                WindowEvent::CloseRequested => return ControlFlow::Exit,
                _ => (),
            }
            _ => (),
        }

        ControlFlow::Wait
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        let (width, height) = (
            NonZeroU32::new(new_size.width).unwrap(),
            NonZeroU32::new(new_size.height).unwrap(),
        );
        self.surface.resize(width, height).unwrap();
    }

    pub fn draw(&mut self) {
        let mut buffer = self.surface.buffer_mut().unwrap();

        let light_color = (255 << 16) | (255 << 8) | 255;
        buffer.fill(light_color);

        buffer.present().unwrap();
    }
}

fn main() -> Result<(), winit::error::EventLoopError> {
    let event_loop = EventLoop::new().unwrap();
    let mut state = State::new(&event_loop);

    event_loop.run(move |event, _, flow| {
        *flow = state.process(event);
    })
}
