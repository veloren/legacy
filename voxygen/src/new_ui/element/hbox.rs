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

pub struct HBox {
    col: Rgba<f32>,
    margin: Size,
    children: Vec<Rc<RefCell<dyn Element>>>,
}

impl HBox {
    pub fn new() -> Self {
        Self {
            col: Rgba::zero(),
            margin: Size::zero(),
            children: Vec::new(),
        }
    }

    pub fn with_color(mut self, col: Rgba<f32>) -> Self {
        self.col = col;
        self
    }

    pub fn with_margin(mut self, margin: Size) -> Self {
        self.margin = margin;
        self
    }

    pub fn push_child<E: Element>(&mut self, element: E) -> Rc<RefCell<E>> {
        let rc = Rc::new(RefCell::new(element));
        self.children.push(rc.clone());
        rc
    }
}

impl Element for HBox {
    fn deep_clone(&self) -> Rc<RefCell<dyn Element>> {
        Rc::new(RefCell::new(self.clone()))
    }

    fn render(&self, renderer: &mut Renderer, rescache: &mut ResCache, bounds: (Vec2<f32>, Vec2<f32>)) {
        draw_rectangle(renderer, rescache, bounds.0, bounds.1, self.col);

        let res = renderer.get_view_resolution().map(|e| e as f32);
        let margin_rel = self.margin.rel * bounds.1 + self.margin.px.map(|e| e as f32) / res;
        let child_bounds = (bounds.0 + margin_rel, bounds.1 - margin_rel * 2.0);
        for (i, child) in self.children.iter().enumerate() {
            child.borrow_mut().render(renderer, rescache, (
                child_bounds.0 + Vec2::new(i as f32 * child_bounds.1.x / self.children.len() as f32, 0.0),
                child_bounds.1 / Vec2::new(self.children.len() as f32, 1.0),
            ));
        }
    }
}

impl Clone for HBox {
    fn clone(&self) -> Self {
        Self {
            col: self.col,
            margin: self.margin,
            children: self.children.iter().map(|c| c.borrow_mut().deep_clone()).collect()
        }
    }
}
