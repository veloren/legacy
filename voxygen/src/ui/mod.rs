extern crate conrod;
extern crate glutin;

mod ui_internal;
mod convert_events;
mod ui_components;

use renderer::Renderer;
use client::Client;
use game::Payloads;
use std::cell::RefCell;

use self::ui_internal::UiInternal;

use glutin:: {
    ElementState,
    MouseButton,
    KeyboardInput,
};

pub struct Ui {
    internal: RefCell<UiInternal>,
}

impl Ui {
    pub fn new(renderer: &mut Renderer, size: [f64; 2]) -> Self {
        Self {
            internal: RefCell::new(UiInternal::new(renderer, size)),
        }
    }

    pub fn render(&self, renderer: &mut Renderer, client: &Client<Payloads>, window_size: &[f64; 2]) {
        self.internal.borrow_mut().render(renderer, &client, window_size);
    }

    pub fn ui_event_keyboard_input(&self, input: KeyboardInput) {
        self.internal.borrow_mut().ui_event_keyboard_input(input);
    }

    pub fn ui_event_window_resize(&self, w: u32, h: u32) {
        self.internal.borrow_mut().ui_event_window_resize(w, h);
    }

    pub fn ui_event_mouse_button(&self, state: ElementState, button: MouseButton) {
        self.internal.borrow_mut().ui_event_mouse_button(state, button);
    }

    pub fn ui_event_mouse_pos(&self, x: f64, y: f64) {
        self.internal.borrow_mut().ui_event_mouse_pos(x, y);
    }

    pub fn ui_event_character(&self, ch: char) {
        self.internal.borrow_mut().ui_event_character(ch);
    }
}
