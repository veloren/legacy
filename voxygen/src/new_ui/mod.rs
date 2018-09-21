// Modules
pub mod elements;
pub mod rescache;
mod primitive;
mod render;
#[cfg(test)]
mod tests;

// Reexports
pub use self::rescache::ResCache;

// Library
use vek::*;

// Local
use renderer::Renderer;

pub trait Element: 'static {
    fn deep_clone(&self) -> Box<dyn Element>;
    fn render(&self, renderer: &mut Renderer, rescache: &mut ResCache, bounds: (Vec2<f32>, Vec2<f32>));
}

pub struct Ui {
    base: Box<Element>,
}

impl Ui {
    pub fn new<T: Element>(base: T) -> Ui {
        Ui { base: Box::new(base) }
    }

    pub fn render(self, renderer: &mut Renderer, res_cache: &mut ResCache) {
        self.base.render(renderer, res_cache, (Vec2::zero(), Vec2::one()));
    }
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
