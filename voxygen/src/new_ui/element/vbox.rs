// Standard
use std::{
    cell::{Cell, RefCell},
    collections::VecDeque,
    rc::Rc,
};

// Library
use vek::*;

// Local
use super::{primitive::draw_rectangle, Bounds, Element, Event, ResCache, Span};
use renderer::Renderer;

#[allow(dead_code)]
pub struct VBox {
    col: Cell<Rgba<f32>>,
    margin: Cell<Vec2<Span>>,
    children: RefCell<VecDeque<Rc<dyn Element>>>,
}

impl VBox {
    #[allow(dead_code)]
    pub fn new() -> Rc<Self> {
        Rc::new(Self {
            col: Cell::new(Rgba::zero()),
            margin: Cell::new(Span::zero()),
            children: RefCell::new(VecDeque::new()),
        })
    }

    #[allow(dead_code)]
    pub fn with_color(self: Rc<Self>, col: Rgba<f32>) -> Rc<Self> {
        self.col.set(col);
        self
    }

    #[allow(dead_code)]
    pub fn with_margin(self: Rc<Self>, margin: Vec2<Span>) -> Rc<Self> {
        self.margin.set(margin);
        self
    }

    #[allow(dead_code)]
    pub fn push_back<E: Element>(&self, child: Rc<E>) -> Rc<E> {
        self.children.borrow_mut().push_back(child.clone());
        child
    }

    #[allow(dead_code)]
    pub fn pop_front(&self) -> Option<Rc<dyn Element>> { self.children.borrow_mut().pop_front() }

    #[allow(dead_code)]
    pub fn get_color(&self) -> Rgba<f32> { self.col.get() }
    #[allow(dead_code)]
    pub fn set_color(&self, col: Rgba<f32>) { self.col.set(col); }

    #[allow(dead_code)]
    pub fn get_margin(&self) -> Vec2<Span> { self.margin.get() }
    #[allow(dead_code)]
    pub fn set_margin(&self, margin: Vec2<Span>) { self.margin.set(margin); }

    fn bounds_for_child(&self, child_index: usize, scr_res: Vec2<f32>, bounds: Bounds) -> Bounds {
        let margin_rel = self.margin.get().map(|e| e.rel) * bounds.1 + self.margin.get().map(|e| e.px as f32) / scr_res;
        let child_bounds = (bounds.0 + margin_rel, bounds.1 - margin_rel * 2.0);
        let child_count = self.children.borrow().len();
        let offs = child_bounds.0 + Vec2::new(0.0, child_index as f32 * child_bounds.1.y / child_count as f32);
        let size = child_bounds.1 / Vec2::new(1.0, child_count as f32);
        (offs, size)
    }
}

impl Element for VBox {
    fn deep_clone(&self) -> Rc<dyn Element> { Rc::new(self.clone()) }

    fn render(&self, renderer: &mut Renderer, rescache: &mut ResCache, bounds: Bounds) {
        draw_rectangle(renderer, rescache, bounds.0, bounds.1, self.col.get());

        let scr_res = renderer.get_view_resolution().map(|e| e as f32);

        for (i, child) in self.children.borrow().iter().enumerate() {
            child.render(renderer, rescache, self.bounds_for_child(i, scr_res, bounds));
        }
    }

    fn handle_event(&self, event: &Event, scr_res: Vec2<f32>, bounds: Bounds) -> bool {
        self.children
            .borrow()
            .iter()
            .enumerate()
            .fold(false, |used, (i, child)| {
                used | child.handle_event(event, scr_res, self.bounds_for_child(i, scr_res, bounds))
            })
    }
}

impl Clone for VBox {
    fn clone(&self) -> Self {
        Self {
            col: self.col.clone(),
            margin: self.margin.clone(),
            children: RefCell::new(self.children.borrow().iter().map(|c| c.deep_clone()).collect()),
        }
    }
}
