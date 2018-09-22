// Modules
pub mod span;
pub mod element;
pub mod rescache;
mod primitive;
mod render;
#[cfg(test)]
mod tests;

// Reexports
pub use self::rescache::ResCache;
pub use self::span::Span;

// Standard
use std::rc::Rc;

// Library
use vek::*;

// Local
use renderer::Renderer;
use self::element::Element;

pub struct Ui {
    base: Rc<dyn Element>,
}

impl Ui {
    pub fn new(base: Rc<dyn Element>) -> Ui {
        Ui { base }
    }

    pub fn render(self, renderer: &mut Renderer, res_cache: &mut ResCache) {
        self.base.render(renderer, res_cache, (Vec2::zero(), Vec2::one()));
    }
}

// Utility types

#[derive(Copy, Clone, Debug)]
pub struct Pos {
    pub rel: Vec2<f32>,
    pub px: Vec2<i16>,
}

impl Pos {
    pub fn rel_and_px(rx: f32, ry: f32, px: i16, py: i16) -> Self {
        Self { rel: Vec2::new(rx, ry), px: Vec2::new(px, py), }
    }

    pub fn rel(x: f32, y: f32) -> Self {
        Self { rel: Vec2::new(x, y), px: Vec2::zero(), }
    }

    pub fn px(x: i16, y: i16) -> Self {
        Self { rel: Vec2::zero(), px: Vec2::new(x, y), }
    }

    pub fn zero() -> Self {
        Self { rel: Vec2::zero(), px: Vec2::zero() }
    }

    fn get_rel(&self) -> Self {
        Self { rel: self.rel, px: Vec2::zero() }
    }

    fn get_px(&self) -> Self {
        Self { rel: Vec2::zero(), px: self.px }
    }
}

impl From<Vec2<f32>> for Pos {
    fn from(v: Vec2<f32>) -> Self { Self::rel(v.x, v.y) }
}

impl From<Vec2<i16>> for Pos {
    fn from(v: Vec2<i16>) -> Self { Self::px(v.x, v.y) }
}

#[derive(Copy, Clone, Debug)]
pub struct Size {
    pub rel: Vec2<f32>,
    pub px: Vec2<i16>,
}

impl Size {
    pub fn rel_and_px(rx: f32, ry: f32, px: i16, py: i16) -> Self {
        Self { rel: Vec2::new(rx, ry), px: Vec2::new(px, py), }
    }

    pub fn rel(x: f32, y: f32) -> Self {
        Self { rel: Vec2::new(x, y), px: Vec2::zero(), }
    }

    pub fn px(x: i16, y: i16) -> Self {
        Self { rel: Vec2::zero(), px: Vec2::new(x, y), }
    }

    pub fn zero() -> Self {
        Self { rel: Vec2::zero(), px: Vec2::zero() }
    }

    pub fn max() -> Self {
        Self { rel: Vec2::one(), px: Vec2::zero(), }
    }

    fn get_rel(&self) -> Self {
        Self { rel: self.rel, px: Vec2::zero() }
    }

    fn get_px(&self) -> Self {
        Self { rel: Vec2::zero(), px: self.px }
    }
}

impl From<Vec2<f32>> for Size {
    fn from(v: Vec2<f32>) -> Self { Self::rel(v.x, v.y) }
}

impl From<Vec2<i16>> for Size {
    fn from(v: Vec2<i16>) -> Self { Self::px(v.x, v.y) }
}

/*
// Example usage
// -------------

fn render(renderer: &mut Renderer, res_cache: &mut ResCache) {
    let ui_font = Font::load("Fantasque Sans Mono").expect("Could not load font");
    let max_msgs = 10;
    let msg_sz = Vec2::new(200, 16);

    let mut chat_box = HBox::new()
        .with_background(Texture::opacity(0.0))
        .with_size(Sz::Px(Vec2::new(msg_sz.x, max_msgs * msg_sz.y)));

    for msg in self.chat_msg[0..max_msgs] {
        chat_box = chat_box.with_child(
            Label::new()
                .with_text(msg)
                .with_font(ui_font)
                .with_color(Rgba::new(1.0, 1.0, 1.0, 1.0))
                .with_border(1.0, Rgba::new(0.0, 0.0, 0.0, 1.0))
        );
    }

    Ui::new(WinBox::new()
        .with_background(Texture::opacity(0.0))
        .with_child_at(Vec2::new(0.0, 0.0), Vec2::new(0.0, 0.0), chat_box)
    ).render(renderer, res_cache);
}
*/
