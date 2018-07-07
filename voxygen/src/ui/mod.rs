extern crate conrod;
extern crate glutin;

mod convert_events;

use conrod::{
    Ui as conrod_ui,
    UiBuilder,
    image::Map,
    color,
    widget::{
        self,
        triangles::Triangle,
    },
    Widget,
    render::Primitives,
    Colorable,
    Sizeable,
    Positionable,
    Borderable,
    Scalar,
    UiCell,
    widget::Id as wid,
    text::font::Id as fid,
    event::Input,
    backend::gfx::Renderer as ConrodRenderer,
    input:: {
        self,
        Key,
    }
};

use glutin:: {
    ElementState,
    MouseButton,
    KeyboardInput,
    VirtualKeyCode,
};

use renderer::Renderer;

use std::collections::HashMap;

pub use gfx_device_gl::Resources as ui_resources;
pub use conrod::gfx_core::handle::ShaderResourceView;

// UI image assets if I understand correctly
pub type ImageMap = Map<(ShaderResourceView<ui_resources, [f32; 4]>, (u32, u32))>;

pub struct Ui {
    conrod_renderer: ConrodRenderer<'static, ui_resources>,
    ui: conrod_ui,
    image_map: ImageMap,
}

impl Ui {
    pub fn new(renderer: &mut Renderer, size: [f64; 2]) -> Self {
        let mut ui = UiBuilder::new(size).build();
        ui.theme.font_id = Some(ui.fonts.insert_from_file("assets/fonts/NotoSans-Regular.ttf").unwrap());

        let image_map = Map::new();

        let color_view = renderer.color_view().clone();
        let mut factory = renderer.factory_mut().clone();

        let conrod_renderer = ConrodRenderer::new(&mut factory, &color_view , 1.0).unwrap();

        Self {
            conrod_renderer,
            ui,
            image_map,
        }
    }

    pub fn render(&mut self, renderer: &mut Renderer, window_size: &[f64; 2]) {
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

    fn generate_widget_id(&mut self) -> widget::Id {
        self.ui.widget_id_generator().next()
    }
}
