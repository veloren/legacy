use conrod::{
    self,
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
    input,
};

use glutin::ElementState;
use glutin::MouseButton;
use renderer::Renderer;

use std::collections::HashMap;

pub use gfx_device_gl::Resources as ui_resources;
pub use conrod::gfx_core::handle::ShaderResourceView;

// UI image assets if I understand correctly
pub type ImageMap = Map<(ShaderResourceView<ui_resources, [f32; 4]>, (u32, u32))>;

pub struct Ui {
    conrodRenderer: ConrodRenderer<'static, ui_resources>,
    ui: conrod_ui,
    image_map: ImageMap,
    fid: Option<fid>,
    ids: HashMap<String, widget::Id>,
}


impl Ui {
    pub fn new(renderer: &mut Renderer, size: [f64; 2]) -> Self {
        let mut ui = UiBuilder::new(size).build();

//        ui.handle_event(Input::Motion(input::Motion::MouseCursor {x: 0.0, y: 0.0}));

        let image_map = Map::new();

        let color_view = renderer.color_view().clone();
        let mut factory = renderer.factory_mut().clone();

        let conrodRenderer = ConrodRenderer::new(&mut factory, &color_view , 1.0).unwrap();

        Self {
            conrodRenderer,
            ui,
            image_map,
            fid: None,
            ids: HashMap::new(),
        }
    }

    pub fn render(&mut self, renderer: &mut Renderer, window_size: &[f64; 2]) {
        self.set_ui();
        self.conrodRenderer.on_resize(renderer.color_view().clone());
        self.conrodRenderer.fill(&mut renderer.encoder_mut(), (window_size[0] as f32 , window_size[1] as f32), 1.0, self.ui.draw(), &self.image_map);
        self.conrodRenderer.draw(&mut renderer.factory_mut().clone(), &mut renderer.encoder_mut(), &self.image_map);
    }

    fn map_mouse(mouse_button: MouseButton) -> input::MouseButton {
        match mouse_button {
            MouseButton::Left => input::MouseButton::Left,
            MouseButton::Right => input::MouseButton::Right,
            MouseButton::Middle => input::MouseButton::Middle,
            MouseButton::Other(0) => input::MouseButton::X1,
            MouseButton::Other(1) => input::MouseButton::X2,
            MouseButton::Other(2) => input::MouseButton::Button6,
            MouseButton::Other(3) => input::MouseButton::Button7,
            MouseButton::Other(4) => input::MouseButton::Button8,
            _ => input::MouseButton::Unknown,
        }
    }

    pub fn ui_event_window_resize(&mut self, w: u32, h: u32) {
        self.ui.handle_event(Input::Resize(w, h));
    }

    pub fn ui_event_mouse_button(&mut self, state: ElementState, button: MouseButton) {
        let event: Input = match state {
            ElementState::Pressed => {
                Input::Press(input::Button::Mouse(Self::map_mouse(button))).into()
            },
            ElementState::Released => {
                Input::Release(input::Button::Mouse(Self::map_mouse(button))).into()
            }
        };

        self.ui.handle_event(event);
    }

    pub fn ui_event_mouse_pos(&mut self, x: f64, y: f64) {
        let dpi = 1.0;
        let tx = |x: Scalar| (x / dpi) - self.ui.win_w / 2.0;
        let ty = |y: Scalar| -((y / dpi) - self.ui.win_h / 2.0);

        let x = tx(x as Scalar);
        let y = ty(y as Scalar);
        let motion = input::Motion::MouseCursor { x, y };
        self.ui.handle_event(Input::Motion(motion).into());
    }

    pub fn add_version_number(&mut self) {
        let fid =self.ui.fonts.insert_from_file("assets/fonts/NotoSans-Regular.ttf").unwrap();
        self.ui.theme.font_id = Some(fid);
        self.fid = Some(fid);
    }

    pub fn generate_widget_id(&mut self) -> widget::Id {
        self.ui.widget_id_generator().next()
    }

    pub fn get_widget_id<T>(&mut self, widget_name: T) -> widget::Id where T: Into<String> {
        let key = widget_name.into();
        if self.ids.contains_key(&key) {
            *self.ids.get(&key).unwrap()
        } else {
            println!("Generated new widget_id: {}", key);
            let id = self.generate_widget_id();
            self.ids.insert(key, id);
            id
        }
    }

    pub fn set_ui(&mut self) {
        let left_text = self.get_widget_id("left_text");
        let range_slider = self.get_widget_id("range_slider");
        let canvas = self.get_widget_id("canvas");
        let oval = self.get_widget_id("oval");
        let font = self.fid.unwrap();

        let mut ui = &mut self.ui.set_widgets();

        widget::Canvas::new().color(color::DARK_CHARCOAL).set(canvas, ui);

        const PAD: conrod::Scalar = 20.0;
        let (ref mut start, ref mut end) = (0.25, 0.75);
        let min = 0.0;
        let max = 1.0;
        for (edge, value) in widget::RangeSlider::new(*start, *end, min, max)
            .color(color::LIGHT_BLUE)
            .padded_w_of(canvas, PAD)
            .h(30.0)
            .mid_top_with_margin_on(canvas, PAD)
            .set(range_slider, ui)
            {
                match edge {
                    widget::range_slider::Edge::Start => *start = value,
                    widget::range_slider::Edge::End => *end = value,
                }
            }

        let range_slider_w = ui.w_of(range_slider).unwrap();
        let w = (*end - *start) * range_slider_w;
        let h = 200.0;
        widget::Oval::fill([w, h])
            .mid_left_with_margin_on(canvas, PAD + *start * range_slider_w)
            .color(color::LIGHT_BLUE)
            .down(50.0)
            .set(oval, ui);

        widget::Text::new(&format!("Version {}", env!("CARGO_PKG_VERSION")))
            .font_id(font)
            .color(color::DARK_CHARCOAL)
            .bottom_left_with_margin(20.0)
            .left_justify()
            .line_spacing(10.0)
            .set(left_text, ui);

    }
}
