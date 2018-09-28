// Modules
pub mod element;
mod primitive;
mod render;
pub mod rescache;
pub mod span;
#[cfg(test)]
mod tests;

// Reexports
pub use self::{rescache::ResCache, span::Span};

// Standard
use std::rc::Rc;

// Library
use vek::*;

// Local
use self::element::Element;
use renderer::Renderer;

pub struct Ui {
    base: Rc<dyn Element>,
}

impl Ui {
    pub fn new(base: Rc<dyn Element>) -> Ui { Ui { base } }

    pub fn render(&self, renderer: &mut Renderer, rescache: &mut ResCache) {
        self.base.render(renderer, rescache, (Vec2::zero(), Vec2::one()));
    }
}
