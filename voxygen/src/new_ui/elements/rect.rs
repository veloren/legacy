// Library
use vek::*;

// Local
use renderer::Renderer;
use super::Element;
use super::ResCache;
use super::primitive::draw_rectangle;

pub struct Rect {
    col: Rgba<f32>,
}

impl Rect {
    pub fn new() -> Rect {
        Rect {
            col: Rgba::zero(),
        }
    }

    pub fn with_color(mut self, col: Rgba<f32>) -> Self {
        self.col = col;
        self
    }
}

impl Element for Rect {
    fn deep_clone(&self) -> Box<dyn Element> {
        Box::new(self.clone())
    }

    fn render(&self, renderer: &mut Renderer, rescache: &mut ResCache, bounds: (Vec2<f32>, Vec2<f32>)) {
        draw_rectangle(renderer, rescache, bounds.0, bounds.1, self.col);
    }
}

impl Clone for Rect {
    fn clone(&self) -> Rect {
        Rect { col: self.col }
    }
}
