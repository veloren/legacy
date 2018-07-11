use gfx_window_glutin;
use glutin;

use glutin::{
    Event as glutinEvent,
    EventsLoop,
    WindowBuilder,
    ContextBuilder,
    GlContext,
    GlRequest,
    GlWindow,
    Window,
    DeviceEvent,
    WindowEvent,
    MouseCursor,
};
use glutin::dpi::{LogicalSize, LogicalPosition};
use glutin::Api::OpenGl;

use renderer::{Renderer, ColorFormat, DepthFormat};

use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::sync::atomic::{AtomicBool, Ordering};

pub enum Event {
    CloseRequest,
    CursorMoved { dx: f64, dy: f64 },
    MouseWheel { dx: f64, dy: f64, modifiers: glutin::ModifiersState },
    KeyboardInput { i: glutin::KeyboardInput, device: glutin::DeviceId },
    Resized { w: u32, h: u32 },
    CursorPosition {x: f64, y: f64},
    MouseButton { state: glutin::ElementState, button: glutin::MouseButton },
    Character { ch: char },
    Raw { event: glutinEvent },
}

pub struct RenderWindow {
    events_loop: RwLock<EventsLoop>,
    gl_window: RwLock<GlWindow>,
    renderer: RwLock<Renderer>,
    cursor_trapped: AtomicBool,
}

impl RenderWindow {
    pub fn new() -> RenderWindow {
        let events_loop = RwLock::new(EventsLoop::new());
        let win_builder = WindowBuilder::new()
            .with_title("Veloren (Voxygen)")
            .with_dimensions(LogicalSize::new(800.0, 500.0))
            .with_maximized(false);

        let ctx_builder = ContextBuilder::new()
            .with_gl(GlRequest::Specific(OpenGl, (3, 2)))
            .with_vsync(true);

        let (gl_window, device, factory, color_view, depth_view) =
            gfx_window_glutin::init::<ColorFormat, DepthFormat>(win_builder, ctx_builder, &events_loop.read().unwrap());

        let rw = RenderWindow {
            events_loop,
            gl_window: RwLock::new(gl_window),
            renderer: RwLock::new(Renderer::new(device, factory, color_view, depth_view)),
            cursor_trapped: AtomicBool::new(true),
        };
        rw.trap_cursor();
        rw
    }

    pub fn handle_events<'a, F: FnMut(Event)>(&self, mut func: F) {
        // We need to mutate these inside the closure, so we take a mutable reference
        let gl_window = &mut self.gl_window.read().unwrap();
        let events_loop = &mut self.events_loop.write().unwrap();

        events_loop.poll_events(|event| {
            match event {
                glutin::Event::DeviceEvent { event, .. } => match event {
                    DeviceEvent::MouseMotion { delta: (dx, dy), .. } => {
                        func(Event::CursorMoved { dx, dy });
                    }
                    _ => {},
                }
                glutin::Event::WindowEvent { event, .. } => match event {
                    WindowEvent::Resized(LogicalSize { width: w, height: h }) => {
                        let mut color_view = self.renderer.read().unwrap().color_view().clone();
                        let mut depth_view = self.renderer.read().unwrap().depth_view().clone();
                        gfx_window_glutin::update_views(
                            &gl_window,
                            &mut color_view,
                            &mut depth_view,
                        );
                        self.renderer.write().unwrap().set_views(color_view, depth_view);
                        func(Event::Resized {
                            w: w as u32,
                            h: h as u32,
                        });
                    },
                    WindowEvent::MouseWheel { delta, modifiers, .. } => {
                        let dx: f64;
                        let dy: f64;
                        match delta {
                            glutin::MouseScrollDelta::LineDelta(x,y) => {
                                dx = f64::from(x) * 8.0;
                                dy = f64::from(y) * 8.0;
                            },
                            glutin::MouseScrollDelta::PixelDelta(LogicalPosition{ x, y }) => {
                                dx = x.into();
                                dy = y.into();
                            },
                        }
                        func(Event::MouseWheel {
                            dx,
                            dy,
                            modifiers,
                        });
                    },
                    WindowEvent::KeyboardInput { device_id, input } => {
                        // keeping the device_id here to allow players using multiple keyboards
                        func(Event::KeyboardInput {
                            device: device_id,
                            i: input,
                        });
                    },
                    WindowEvent::MouseInput { state, button, .. } => {
                        if button == glutin::MouseButton::Left {
                            self.trap_cursor();
                        }

                        func(Event::MouseButton { state, button });
                    },
                    WindowEvent::CloseRequested => func(Event::CloseRequest),

                    WindowEvent::Focused(is_focused) => {
                            self.cursor_trapped.store(is_focused, Ordering::Relaxed);
                    },
                    WindowEvent::CursorMoved { position, .. } => {
                        func(Event::CursorPosition {x: position.x, y: position.y });
                    },
                    WindowEvent::ReceivedCharacter(ch) => {
                        func(Event::Character { ch });
                    },
                    _ => {},
                },
                _ => {
                    func(Event::Raw {event});
                },
            }
        });
    }

    pub fn trap_cursor(&self) {
        self.gl_window.read().unwrap().hide_cursor(true);
        self.gl_window.read().unwrap().grab_cursor(true).expect("Could not grab cursor!");
        self.cursor_trapped.store(true, Ordering::Relaxed);
    }

    pub fn untrap_cursor(&self) {
        self.gl_window.read().unwrap().hide_cursor(false);
        self.gl_window.read().unwrap().grab_cursor(false).expect("Could not ungrab cursor!");
        self.cursor_trapped.store(false, Ordering::Relaxed);
    }

    pub fn swap_buffers(&self) {
        self.gl_window.read().unwrap().swap_buffers().expect("Failed to swap window buffers");
    }

    pub fn get_size(&self) -> [f64; 2] {
        let window = self.gl_window.read().unwrap();
        match window.get_inner_size() {
            Some(LogicalSize{ width: w, height: h }) => [w as f64, h as f64],
            None => [0.0, 0.0]
        }
    }

    #[allow(dead_code)] pub fn renderer(&self) -> RwLockReadGuard<Renderer> { self.renderer.read().unwrap() }
    #[allow(dead_code)] pub fn renderer_mut(&self) -> RwLockWriteGuard<Renderer> { self.renderer.write().unwrap() }

    #[allow(dead_code)] pub fn cursor_trapped(&self) -> &AtomicBool { &self.cursor_trapped }
}
