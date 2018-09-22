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

    pub fn render(&self, renderer: &mut Renderer, rescache: &mut ResCache) {
        self.base.render(renderer, rescache, (Vec2::zero(), Vec2::one()));
    }
}
