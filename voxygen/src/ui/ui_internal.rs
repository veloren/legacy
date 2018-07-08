extern crate fps_counter;

use renderer::Renderer;
use client::Client;

use ::ui::convert_events;
use ::game::Payloads;

use conrod::{
    UiBuilder,
    Ui as conrod_ui,
    backend::gfx::Renderer as ConrodRenderer,
    image::Map,
    event::Input,
    widget,
    UiCell,
};

use ::ui::ui_components::{
    self,
    UiState,
};

use ::glutin::{
    KeyboardInput,
    MouseButton,
    ElementState,
};

pub use gfx_device_gl::Resources as ui_resources;
pub use conrod::gfx_core::handle::ShaderResourceView;
pub type ImageMap = Map<(ShaderResourceView<ui_resources, [f32; 4]>, (u32, u32))>;

pub struct UiInternal {
    conrod_renderer: ConrodRenderer<'static, ui_resources>,
    ui: conrod_ui,
    image_map: ImageMap,
    pub fps: fps_counter::FPSCounter,
    state: UiState,
}

impl UiInternal {
    pub fn new(renderer: &mut Renderer, size: [f64; 2]) -> Self {
        let mut ui = UiBuilder::new(size).build();
        ui.theme.font_id = Some(ui.fonts.insert_from_file("data/assets/fonts/NotoSans-Regular.ttf").unwrap());

        let image_map = Map::new();

        let color_view = renderer.color_view().clone();
        let mut factory = renderer.factory_mut().clone();

        let conrod_renderer = ConrodRenderer::new(&mut factory, &color_view , 1.0).unwrap();


        Self {
            conrod_renderer,
            ui,
            image_map,
            fps: fps_counter::FPSCounter::new(),
            state: UiState::normal_game(),
        }
    }

    pub fn render(&mut self, renderer: &mut Renderer, client: &Client<Payloads>, window_size: &[f64; 2]) {
        ui_components::render(self);
        self.conrod_renderer.on_resize(renderer.color_view().clone());
        self.conrod_renderer.fill(&mut renderer.encoder_mut(), (window_size[0] as f32 , window_size[1] as f32), 1.0, self.ui.draw(), &self.image_map);
        self.conrod_renderer.draw(&mut renderer.factory_mut().clone(), &mut renderer.encoder_mut(), &self.image_map);
    }

    pub fn ui_event_keyboard_input(&mut self, input: KeyboardInput) {
        if let Some(event) =  convert_events::convert_keycode(input) {
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

    pub fn generate_widget_id(&mut self) -> widget::Id {
        self.ui.widget_id_generator().next()
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
        self.state
    }
}