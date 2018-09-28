// Modules
pub mod element;
mod primitive;
mod render;
pub mod rescache;
pub mod span;
#[cfg(test)]
mod tests;

// Reexports
pub use self::span::Span;

// Standard
use std::rc::Rc;

// Library
use vek::*;

// Local
use self::{element::Element, rescache::ResCache};
use renderer::Renderer;

pub struct Ui {
    base: Rc<dyn Element>,
    rescache: ResCache,
}

impl Ui {
    pub fn new(base: Rc<dyn Element>) -> Ui {
        Ui {
            base,
            rescache: ResCache::new(),
        }
    }

    pub fn render(&mut self, renderer: &mut Renderer) {
        self.base
            .render(renderer, &mut self.rescache, (Vec2::zero(), Vec2::one()));
    }
}
