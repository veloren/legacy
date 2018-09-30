// Standard
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

// Library
use vek::*;

// Local
use super::{primitive::draw_rectangle, Bounds, Element, Event, ResCache, Span};
use renderer::Renderer;

pub struct WinBoxChild {
    offset: Vec2<Span>,
    anchor: Vec2<Span>,
    size: Vec2<Span>,
    element: Rc<dyn Element>,
}

#[allow(dead_code)]
pub struct WinBox {
    col: Cell<Rgba<f32>>,
    children: RefCell<Vec<WinBoxChild>>,
}

impl WinBox {
    #[allow(dead_code)]
    pub fn new() -> Rc<Self> {
        Rc::new(Self {
            col: Cell::new(Rgba::zero()),
            children: RefCell::new(Vec::new()),
        })
    }

    #[allow(dead_code)]
    pub fn with_color(self: Rc<Self>, col: Rgba<f32>) -> Rc<Self> {
        self.col.set(col);
        self
    }

    #[allow(dead_code)]
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

    #[allow(dead_code)]
    pub fn clone_all(&self) -> Rc<Self> { Rc::new(self.clone()) }

    fn bounds_for_child(&self, child: &WinBoxChild, scr_res: Vec2<f32>, bounds: Bounds) -> Bounds {
        let size = child.size.map(|e| e.rel) * bounds.1 + child.size.map(|e| e.px as f32) / scr_res;
        let offs = child.offset.map(|e| e.rel) * bounds.1 - child.anchor.map(|e| e.rel) * bounds.1 * size
            + (child.offset.map(|e| e.px) - child.anchor.map(|e| e.px)).map(|e| e as f32) / scr_res;
        (offs, size)
    }
}

impl Element for WinBox {
    fn deep_clone(&self) -> Rc<dyn Element> { self.clone_all() }

    fn render(&self, renderer: &mut Renderer, rescache: &mut ResCache, bounds: Bounds) {
        draw_rectangle(renderer, rescache, bounds.0, bounds.1, self.col.get());

        let scr_res = renderer.get_view_resolution().map(|e| e as f32);

        for child in self.children.borrow().iter() {
            child
                .element
                .render(renderer, rescache, self.bounds_for_child(child, scr_res, bounds));
        }
    }

    fn handle_event(&self, event: &Event, scr_res: Vec2<f32>, bounds: Bounds) -> bool {
        self.children.borrow().iter().fold(false, |used, child| {
            used | child
                .element
                .handle_event(event, scr_res, self.bounds_for_child(child, scr_res, bounds))
        })
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
                    })
                    .collect(),
            ),
        }
    }
}
