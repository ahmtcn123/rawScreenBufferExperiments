use cursor::{Boundaries, Cursor, Font};
use screen::{Color, ScreenBuffer};
use window::Window as Cwindow;
use std::{
    fs,
    rc::Rc,
    sync::{atomic::AtomicBool, RwLock},
    thread,
};
use std::{
    num::NonZeroU32,
    sync::{Arc, Mutex},
};
use winit::window::CustomCursor;
use winit::window::Window;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

mod cursor;
mod screen;
mod winit_app;
mod window;

fn main() {
    let event_loop = EventLoop::new().unwrap();

    let mut screen = ScreenBuffer::new(500, 500);
    let mut draw_requested = true;
    /* let screen = ScreenBuffer::new(500, 500); */

    let file = fs::read("./fira_code.ttf".to_string()).unwrap();
    let font = Font::from_bytes(file, 18.0);

    /* let mut cursor = Cursor::new(font, Boundaries { start_x: 15, start_y: 15, width: screen.width, height: screen.height }); */
    let mut cwindow = RwLock::new(Cwindow::new(font, (15, 15), 500, 500));




    let mut app = winit_app::WinitAppBuilder::with_init(|elwt| {
        let window = {
            let window = elwt.create_window(Window::default_attributes());
            Rc::new(window.unwrap())
        };
        let context = softbuffer::Context::new(window.clone()).unwrap();
        let surface = softbuffer::Surface::new(&context, window.clone()).unwrap();

        (window, surface)
    })
    .with_event_handler(|state, event, elwt| {
        let (window, surface) = state;
        elwt.set_control_flow(ControlFlow::Wait);

        match event {
            Event::WindowEvent {
                window_id,
                event: WindowEvent::RedrawRequested,
            } if window_id == window.id() => {
                let (width, height) = {
                    let size = window.inner_size();
                    (size.width, size.height)
                };
                surface
                    .resize(
                        NonZeroU32::new(width).unwrap(),
                        NonZeroU32::new(height).unwrap(),
                    )
                    .unwrap();

                    if screen.width != width as usize
                    || screen.height != height as usize
                {
                    screen.resize(width as usize, height as usize);
                }
                draw_requested = false;
                let mut buffer = surface.buffer_mut().unwrap();
                let buffer_size = buffer.len();
                let screen_size = screen.buffer.len();
                cwindow.read().unwrap().render_on_screen(&mut screen);
                buffer[..screen_size].copy_from_slice(&screen.buffer);
                buffer.present().unwrap();
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => {
                elwt.exit();
            }
            Event::WindowEvent {
                window_id,
                event: WindowEvent::Resized(size),
            } => {
                /*                 event_screen
                .lock()
                .unwrap()
                .resize(size.width as usize, size.height as usize); */
                println!("Window Resize: {:?}", size);
            }
            Event::WindowEvent {
                window_id,
                event:
                    WindowEvent::MouseInput {
                        device_id,
                        state,
                        button,
                    },
            } => {
                println!("Mouse Input: {:?}", button);
                println!("Mouse State: {:?}", state);
                println!("Mouse Device ID: {:?}", device_id);

                cwindow.write().unwrap().cursor.println(&format!("Mouse Input: {:?}", button));
                cwindow.write().unwrap().cursor.println(&format!("Mouse State: {:?}", state));
                cwindow.write().unwrap().cursor.println(&format!("Mouse Device ID: {:?}", device_id));
            }
            e => {
                println!("Other Events: {:?}", e);
            }
        }
    });

    event_loop.run_app(&mut app).unwrap();
}
