// Standard
use std::{
    cell::{Cell, RefCell},
    collections::VecDeque,
    rc::Rc,
};

// Library
use vek::*;

// Local
use super::{primitive::draw_rectangle, Element, ResCache, Span, Bounds};
use renderer::Renderer;

#[allow(dead_code)]
pub struct HBox {
    col: Cell<Rgba<f32>>,
    margin: Cell<Vec2<Span>>,
    children: RefCell<VecDeque<Rc<dyn Element>>>,
}

impl HBox {
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

    #[allow(dead_code)]
    pub fn clone_all(&self) -> Rc<Self> { Rc::new(self.clone()) }
}

impl Element for HBox {
    fn deep_clone(&self) -> Rc<dyn Element> { self.clone_all() }

    fn render(&self, renderer: &mut Renderer, rescache: &mut ResCache, bounds: Bounds) {
        draw_rectangle(renderer, rescache, bounds.0, bounds.1, self.col.get());

        let view_res = renderer.get_view_resolution().map(|e| e as f32);
        let margin_rel =
            self.margin.get().map(|e| e.rel) * bounds.1 + self.margin.get().map(|e| e.px as f32) / view_res;
        let child_bounds = (bounds.0 + margin_rel, bounds.1 - margin_rel * 2.0);

        let children = self.children.borrow();
        for (i, child) in children.iter().enumerate() {
            child.render(
                renderer,
                rescache,
                (
                    child_bounds.0 + Vec2::new(i as f32 * child_bounds.1.x / children.len() as f32, 0.0),
                    child_bounds.1 / Vec2::new(children.len() as f32, 1.0),
                ),
            );
        }
    }
}

impl Clone for HBox {
    fn clone(&self) -> Self {
        Self {
            col: self.col.clone(),
            margin: self.margin.clone(),
            children: RefCell::new(self.children.borrow().iter().map(|c| c.deep_clone()).collect()),
        }
    }
}
