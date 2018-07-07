extern crate conrod;
extern crate glutin;

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
    conrodRenderer: ConrodRenderer<'static, ui_resources>,
    ui: conrod_ui,
    image_map: ImageMap,
    fid: Option<fid>,
    ids: HashMap<String, widget::Id>,
    start: f64,
    end: f64,
}

impl Ui {
    pub fn new(renderer: &mut Renderer, size: [f64; 2]) -> Self {
        let ui = UiBuilder::new(size).build();
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
            start: 0.25,
            end: 0.75,
        }
    }

    pub fn render(&mut self, renderer: &mut Renderer, window_size: &[f64; 2]) {
        self.set_ui();
        self.conrodRenderer.on_resize(renderer.color_view().clone());
        self.conrodRenderer.fill(&mut renderer.encoder_mut(), (window_size[0] as f32 , window_size[1] as f32), 1.0, self.ui.draw(), &self.image_map);
        self.conrodRenderer.draw(&mut renderer.factory_mut().clone(), &mut renderer.encoder_mut(), &self.image_map);
    }

    pub fn map_key(keycode: VirtualKeyCode) -> input::keyboard::Key {

        match keycode {
            VirtualKeyCode::Key0 => Key::D0,
            VirtualKeyCode::Key1 => Key::D1,
            VirtualKeyCode::Key2 => Key::D2,
            VirtualKeyCode::Key3 => Key::D3,
            VirtualKeyCode::Key4 => Key::D4,
            VirtualKeyCode::Key5 => Key::D5,
            VirtualKeyCode::Key6 => Key::D6,
            VirtualKeyCode::Key7 => Key::D7,
            VirtualKeyCode::Key8 => Key::D8,
            VirtualKeyCode::Key9 => Key::D9,
            VirtualKeyCode::A => Key::A,
            VirtualKeyCode::B => Key::B,
            VirtualKeyCode::C => Key::C,
            VirtualKeyCode::D => Key::D,
            VirtualKeyCode::E => Key::E,
            VirtualKeyCode::F => Key::F,
            VirtualKeyCode::G => Key::G,
            VirtualKeyCode::H => Key::H,
            VirtualKeyCode::I => Key::I,
            VirtualKeyCode::J => Key::J,
            VirtualKeyCode::K => Key::K,
            VirtualKeyCode::L => Key::L,
            VirtualKeyCode::M => Key::M,
            VirtualKeyCode::N => Key::N,
            VirtualKeyCode::O => Key::O,
            VirtualKeyCode::P => Key::P,
            VirtualKeyCode::Q => Key::Q,
            VirtualKeyCode::R => Key::R,
            VirtualKeyCode::S => Key::S,
            VirtualKeyCode::T => Key::T,
            VirtualKeyCode::U => Key::U,
            VirtualKeyCode::V => Key::V,
            VirtualKeyCode::W => Key::W,
            VirtualKeyCode::X => Key::X,
            VirtualKeyCode::Y => Key::Y,
            VirtualKeyCode::Z => Key::Z,
            VirtualKeyCode::Apostrophe => Key::Unknown,
            VirtualKeyCode::Backslash => Key::Backslash,
            VirtualKeyCode::Back => Key::Backspace,
            // K::CapsLock => Key::CapsLock,
            VirtualKeyCode::Delete => Key::Delete,
            VirtualKeyCode::Comma => Key::Comma,
            VirtualKeyCode::Down => Key::Down,
            VirtualKeyCode::End => Key::End,
            VirtualKeyCode::Return => Key::Return,
            VirtualKeyCode::Equals => Key::Equals,
            VirtualKeyCode::Escape => Key::Escape,
            VirtualKeyCode::F1 => Key::F1,
            VirtualKeyCode::F2 => Key::F2,
            VirtualKeyCode::F3 => Key::F3,
            VirtualKeyCode::F4 => Key::F4,
            VirtualKeyCode::F5 => Key::F5,
            VirtualKeyCode::F6 => Key::F6,
            VirtualKeyCode::F7 => Key::F7,
            VirtualKeyCode::F8 => Key::F8,
            VirtualKeyCode::F9 => Key::F9,
            VirtualKeyCode::F10 => Key::F10,
            VirtualKeyCode::F11 => Key::F11,
            VirtualKeyCode::F12 => Key::F12,
            VirtualKeyCode::F13 => Key::F13,
            VirtualKeyCode::F14 => Key::F14,
            VirtualKeyCode::F15 => Key::F15,
            // K::F16 => Key::F16,
            // K::F17 => Key::F17,
            // K::F18 => Key::F18,
            // K::F19 => Key::F19,
            // K::F20 => Key::F20,
            // K::F21 => Key::F21,
            // K::F22 => Key::F22,
            // K::F23 => Key::F23,
            // K::F24 => Key::F24,
            // Possibly next code.
            // K::F25 => Key::Unknown,
            VirtualKeyCode::Numpad0 => Key::NumPad0,
            VirtualKeyCode::Numpad1 => Key::NumPad1,
            VirtualKeyCode::Numpad2 => Key::NumPad2,
            VirtualKeyCode::Numpad3 => Key::NumPad3,
            VirtualKeyCode::Numpad4 => Key::NumPad4,
            VirtualKeyCode::Numpad5 => Key::NumPad5,
            VirtualKeyCode::Numpad6 => Key::NumPad6,
            VirtualKeyCode::Numpad7 => Key::NumPad7,
            VirtualKeyCode::Numpad8 => Key::NumPad8,
            VirtualKeyCode::Numpad9 => Key::NumPad9,
            VirtualKeyCode::NumpadComma => Key::NumPadDecimal,
            VirtualKeyCode::Divide => Key::NumPadDivide,
            VirtualKeyCode::Multiply => Key::NumPadMultiply,
            VirtualKeyCode::Subtract => Key::NumPadMinus,
            VirtualKeyCode::Add => Key::NumPadPlus,
            VirtualKeyCode::NumpadEnter => Key::NumPadEnter,
            VirtualKeyCode::NumpadEquals => Key::NumPadEquals,
            VirtualKeyCode::LShift => Key::LShift,
            VirtualKeyCode::LControl => Key::LCtrl,
            VirtualKeyCode::LAlt => Key::LAlt,
            VirtualKeyCode::LMenu => Key::LGui,
            VirtualKeyCode::RShift => Key::RShift,
            VirtualKeyCode::RControl => Key::RCtrl,
            VirtualKeyCode::RAlt => Key::RAlt,
            VirtualKeyCode::RMenu => Key::RGui,
            // Map to backslash?
            // K::GraveAccent => Key::Unknown,
            VirtualKeyCode::Home => Key::Home,
            VirtualKeyCode::Insert => Key::Insert,
            VirtualKeyCode::Left => Key::Left,
            VirtualKeyCode::LBracket => Key::LeftBracket,
            // K::Menu => Key::Menu,
            VirtualKeyCode::Minus => Key::Minus,
            VirtualKeyCode::Numlock => Key::NumLockClear,
            VirtualKeyCode::PageDown => Key::PageDown,
            VirtualKeyCode::PageUp => Key::PageUp,
            VirtualKeyCode::Pause => Key::Pause,
            VirtualKeyCode::Period => Key::Period,
            // K::PrintScreen => Key::PrintScreen,
            VirtualKeyCode::Right => Key::Right,
            VirtualKeyCode::RBracket => Key::RightBracket,
            // K::ScrollLock => Key::ScrollLock,
            VirtualKeyCode::Semicolon => Key::Semicolon,
            VirtualKeyCode::Slash => Key::Slash,
            VirtualKeyCode::Space => Key::Space,
            VirtualKeyCode::Tab => Key::Tab,
            VirtualKeyCode::Up => Key::Up,
            // K::World1 => Key::Unknown,
            // K::World2 => Key::Unknown,
            _ => Key::Unknown,
        }
    }

    pub fn ui_event_keyboard_input(&mut self, input: KeyboardInput) {
        if let Some(event) = input.virtual_keycode.map(|key| {
            match input.state {
                ElementState::Pressed =>
                    Input::Press(input::Button::Keyboard(Self::map_key(key))),
                ElementState::Released =>
                    Input::Release(input::Button::Keyboard(Self::map_key(key))),
            }
        }) {
            self.ui.handle_event(event);
        }
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
        let fid =self.ui.fonts.insert_from_file("data/assets/fonts/NotoSans-Regular.ttf").unwrap();
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
        let mut ui = &mut self.ui.set_widgets();
        let font = self.fid.unwrap();

        widget::Text::new(&format!("Version {}", env!("CARGO_PKG_VERSION")))
            .font_id(font)
            .color(color::DARK_CHARCOAL)
            .bottom_left_with_margin(20.0)
            .left_justify()
            .line_spacing(10.0)
            .set(left_text, ui);
    }
}
