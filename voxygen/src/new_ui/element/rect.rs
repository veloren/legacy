// Standard
use std::{
    rc::Rc,
    cell::RefCell
};

// Library
use vek::*;

// Local
use renderer::Renderer;
use super::{
    Element,
    ResCache,
    Pos,
    Size,
};
use super::primitive::draw_rectangle;

#[derive(Clone)]
pub struct Rect {
    col: Rgba<f32>,
}

impl Rect {
    pub fn new() -> Self {
        Self {
            col: Rgba::zero(),
        }
    }

    pub fn with_color(mut self, col: Rgba<f32>) -> Self {
        self.col = col;
        self
    }
}

impl Element for Rect {
    fn deep_clone(&self) -> Rc<RefCell<dyn Element>> {
        Rc::new(RefCell::new(self.clone()))
    }

    fn render(&self, renderer: &mut Renderer, rescache: &mut ResCache, bounds: (Vec2<f32>, Vec2<f32>)) {
        draw_rectangle(renderer, rescache, bounds.0, bounds.1, self.col);
    }
}
