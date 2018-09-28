// Standard
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

// Library
use vek::*;

// Local
use super::{primitive::draw_rectangle, Element, ResCache, Span};
use renderer::Renderer;

pub struct WinBoxChild {
    offset: Vec2<Span>,
    anchor: Vec2<Span>,
    size: Vec2<Span>,
    element: Rc<dyn Element>,
}

pub struct WinBox {
    col: Cell<Rgba<f32>>,
    children: RefCell<Vec<WinBoxChild>>,
}

impl WinBox {
    pub fn new() -> Rc<Self> {
        Rc::new(Self {
            col: Cell::new(Rgba::zero()),
            children: RefCell::new(Vec::new()),
        })
    }

    pub fn with_color(self: Rc<Self>, col: Rgba<f32>) -> Rc<Self> {
        self.col.set(col);
        self
    }

    pub fn add_child_at<E: Element>(
        &self,
        offset: Vec2<Span>,
        anchor: Vec2<Span>,
        size: Vec2<Span>,
        child: Rc<E>,
    ) -> Rc<E> {
        self.children.borrow_mut().push(WinBoxChild {
            offset,
            anchor,
            size,
            element: child.clone(),
        });
        child
    }

    pub fn clone_all(&self) -> Rc<Self> { Rc::new(self.clone()) }
}

impl Element for WinBox {
    fn deep_clone(&self) -> Rc<dyn Element> { self.clone_all() }

    fn render(&self, renderer: &mut Renderer, rescache: &mut ResCache, bounds: (Vec2<f32>, Vec2<f32>)) {
        draw_rectangle(renderer, rescache, bounds.0, bounds.1, self.col.get());

        let res = renderer.get_view_resolution().map(|e| e as f32);

        for WinBoxChild {
            offset,
            anchor,
            size,
            element,
        } in self.children.borrow().iter()
        {
            let size = size.map(|e| e.rel) * bounds.1 + size.map(|e| e.px as f32) / res;
            element.render(
                renderer,
                rescache,
                (
                    offset.map(|e| e.rel) * bounds.1 - anchor.map(|e| e.rel) * bounds.1 * size
                        + (offset.map(|e| e.px) - anchor.map(|e| e.px)).map(|e| e as f32) / res,
                    size,
                ),
            );
        }
    }
}

impl Clone for WinBox {
    fn clone(&self) -> Self {
        Self {
            col: self.col.clone(),
            children: RefCell::new(
                self.children
                    .borrow()
                    .iter()
                    .map(|c| WinBoxChild {
                        offset: c.offset,
                        anchor: c.anchor,
                        size: c.size,
                        element: c.element.deep_clone(),
                    }).collect(),
            ),
        }
    }
}
