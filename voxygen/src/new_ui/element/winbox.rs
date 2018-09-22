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

pub struct WinBoxChild {
    offset: Pos,
    anchor: Pos,
    size: Size,
    element: Rc<RefCell<dyn Element>>,
}

pub struct WinBox {
    col: Rgba<f32>,
    children: Vec<WinBoxChild>,
}

impl WinBox {
    pub fn new() -> Self {
        Self {
            col: Rgba::zero(),
            children: Vec::new(),
        }
    }

    pub fn with_color(mut self, col: Rgba<f32>) -> Self {
        self.col = col;
        self
    }

    pub fn add_child_at<E: Element>(&mut self, offset: Pos, anchor: Pos, size: Size, element: E) -> Rc<RefCell<E>> {
        let rc = Rc::new(RefCell::new(element));
        self.children.push(WinBoxChild {
            offset,
            anchor,
            size,
            element: rc.clone(),
        });
        rc
    }
}

impl Element for WinBox {
    fn deep_clone(&self) -> Rc<RefCell<dyn Element>> {
        Rc::new(RefCell::new(self.clone()))
    }

    fn render(&self, renderer: &mut Renderer, rescache: &mut ResCache, bounds: (Vec2<f32>, Vec2<f32>)) {
        draw_rectangle(renderer, rescache, bounds.0, bounds.1, self.col);

        let res = renderer.get_view_resolution().map(|e| e as f32);

        for WinBoxChild { offset, anchor, size, element } in self.children.iter() {
            let size = size.rel * bounds.1 + size.px.map(|e| e as f32) / res;
            element.borrow_mut().render(renderer, rescache, (
                offset.rel * bounds.1 - anchor.rel * bounds.1 * size + (offset.px - anchor.px).map(|e| e as f32) / res,
                size,
            ));
        }
    }
}

impl Clone for WinBox {
    fn clone(&self) -> Self {
        Self {
            col: self.col,
            children: self.children.iter().map(|c| WinBoxChild {
                offset: c.offset,
                anchor: c.anchor,
                size: c.size,
                element: c.element.borrow_mut().deep_clone(),
            }).collect()
        }
    }
}
