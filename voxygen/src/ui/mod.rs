extern crate conrod;
extern crate glutin;
extern crate fps_counter;

mod convert_events;
mod ui_components;

use renderer::Renderer;
use client::Client;
use game::Payloads;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::mpsc::{self, Sender, Receiver};

use self::ui_components::{
    UiState,
    MAX_CHAT_LINES,
};

use conrod::{
    UiBuilder,
    Ui as conrod_ui,
    backend::gfx::Renderer as ConrodRenderer,
    image::Map,
    event::Input,
    widget,
    UiCell
};

pub use gfx_device_gl::Resources as ui_resources;
pub use conrod::gfx_core::handle::ShaderResourceView;
pub type ImageMap = Map<(ShaderResourceView<ui_resources, [f32; 4]>, (u32, u32))>;

use glutin:: {
    ElementState,
    MouseButton,
    KeyboardInput,
};

pub enum UiInternalEvent {
    UpdateChatText(String),
    NewChatMessage(String, String),
    SendChat,
}

pub struct Ui {
    conrod_renderer: ConrodRenderer<'static, ui_resources>,
    ui: conrod_ui,
    image_map: ImageMap,
    pub fps: fps_counter::FPSCounter,
    state: UiState,
    ids: HashMap<String, widget::Id>,
    event_tx: Sender<UiInternalEvent>,
    event_rx: Receiver<UiInternalEvent>
}

impl Ui {
    pub fn new(renderer: &mut Renderer, size: [f64; 2], client: &Client<Payloads>) -> Self {
        let mut ui = UiBuilder::new(size).build();
        let mut factory = renderer.factory_mut().clone();
        let color_view = renderer.color_view().clone();
        let conrod_renderer = ConrodRenderer::new(&mut factory, &color_view, 1.0).unwrap();
        let image_map = Map::new();
        let (tx, rx) = mpsc::channel();

        let tx2 = tx.clone();
        client.callbacks().set_recv_chat_msg(move |alias, msg| {
            if tx2.send(UiInternalEvent::NewChatMessage(alias.to_string(), msg.to_string())).is_err() {
                panic!("Could not send event to ui");
            }
        });

        ui.theme.font_id = Some(ui.fonts.insert_from_file("data/assets/fonts/NotoSans-Regular.ttf").unwrap());

        Self {
            conrod_renderer,
            ui,
            image_map,
            fps: fps_counter::FPSCounter::new(),
            state: UiState::normal_game(),
            ids: HashMap::new(),
            event_tx: tx,
            event_rx: rx,
        }
    }

    pub fn render(&mut self, renderer: &mut Renderer, client: &Client<Payloads>, window_size: &[f64; 2]) {
        self.update_internal_event(&client);
        ui_components::render(self);
        self.conrod_renderer.on_resize(renderer.color_view().clone());
        self.conrod_renderer.fill(&mut renderer.encoder_mut(), (window_size[0] as f32, window_size[1] as f32), 1.0, self.ui.draw(), &self.image_map);
        self.conrod_renderer.draw(&mut renderer.factory_mut().clone(), &mut renderer.encoder_mut(), &self.image_map);
    }

    pub fn ui_event_keyboard_input(&mut self, input: KeyboardInput) {
        if let Some(event) = convert_events::convert_keycode(input) {
            self.ui.handle_event(event);
        }
    }

    pub fn ui_event_window_resize(&mut self, w: u32, h: u32) {
        self.ui.handle_event(Input::Resize(w, h));
    }

    pub fn ui_event_mouse_button(&mut self, state: ElementState, button: MouseButton) {
        self.ui.handle_event(convert_events::convert_mousebutton(state, button));
    }

    pub fn ui_event_mouse_pos(&mut self, x: f64, y: f64) {
        self.ui.handle_event(convert_events::convert_mouse_pos(x, y, self.ui.win_w, self.ui.win_h));
    }

    pub fn ui_event_character(&mut self, ch: char) {
        self.ui.handle_event(convert_events::convert_character(ch));
    }

    fn generate_widget_id(&mut self) -> widget::Id {
        self.ui.widget_id_generator().next()
    }

    pub fn get_widget_id<T>(&mut self, widget_name: T) -> widget::Id where T: Into<String> {
        let key = widget_name.into();
        if self.ids.contains_key(&key) {
            self.ids[&key]
        } else {
            println!("Generated new widget_id: {}", key);
            let id = self.generate_widget_id();
            self.ids.insert(key, id);
            id
        }
    }

    pub fn get_ui_cell(&mut self) -> UiCell {
        self.ui.set_widgets()
    }

    pub fn get_width(&self) -> f64 {
        self.ui.win_w
    }

    pub fn get_height(&self) -> f64 {
        self.ui.win_h
    }

    pub fn get_state(&self) -> UiState {
        self.state.clone()
    }

    pub fn set_show_chat(&mut self, show: bool) {
        self.state.show_chat = show;
    }

    pub fn get_show_chat(&self) -> bool {
        self.state.show_chat.clone()
    }

    pub fn get_event_tx(&self) -> Sender<UiInternalEvent> {
        self.event_tx.clone()
    }

    pub fn widget_events<T>(&self, id: widget::Id, fnc: T) where T: Fn(conrod::event::Widget){
        for widget_event in self.ui.widget_input(id).events() {
            fnc(widget_event);
        }
    }

    fn update_internal_event(&mut self, client: &Client<Payloads>) {
        for event in self.event_rx.try_iter() {
            match event {
                UiInternalEvent::UpdateChatText(edit) => {
                    self.state.chat_message = edit;
                },
                UiInternalEvent::NewChatMessage(alias, msg) => {
                    if self.state.chat_lines.len() >= MAX_CHAT_LINES {
                        self.state.chat_lines.pop_back();
                    }

                    self.state.chat_lines.push_front((alias, msg));
                },
                UiInternalEvent::SendChat => {
                    if self.state.chat_message.len() != 0 {
                        client.send_chat_msg(self.state.chat_message.clone());
                        self.state.chat_message.clear();
                    }
                },
            }
        }
    }
}
