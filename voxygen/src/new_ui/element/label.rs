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
use super::primitive::draw_text;

#[derive(Clone)]
pub struct Label {
    text: Option<String>,
    col: Rgba<f32>,
    size: Size,
}

impl Label {
    pub fn new() -> Self {
        Self {
            text: None,
            col: Rgba::zero(),
            size: Size::px(16, 16),
        }
    }

    pub fn with_text(mut self, text: String) -> Self {
        self.text = Some(text);
        self
    }

    pub fn with_color(mut self, col: Rgba<f32>) -> Self {
        self.col = col;
        self
    }

    pub fn with_size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }
}

impl Element for Label {
    fn deep_clone(&self) -> Rc<RefCell<dyn Element>> {
        Rc::new(RefCell::new(self.clone()))
    }

    fn render(&self, renderer: &mut Renderer, rescache: &mut ResCache, bounds: (Vec2<f32>, Vec2<f32>)) {
        if let Some(text) = &self.text {
            let res = renderer.get_view_resolution().map(|e| e as f32);
            let sz = self.size.rel * res.map(|e| e as f32) + self.size.px.map(|e| e as f32);
            draw_text(renderer, rescache, text, bounds.0, sz, self.col);
        }
    }
}
