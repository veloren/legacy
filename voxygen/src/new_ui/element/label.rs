// Standard
use std::{
    cell::{Cell, Ref, RefCell},
    rc::Rc,
};

// Library
use vek::*;

// Local
use super::{primitive::draw_text, Element, ResCache, Span};
use renderer::Renderer;

#[allow(dead_code)]
#[derive(Clone)]
pub struct Label {
    text: RefCell<Option<String>>,
    col: Cell<Rgba<f32>>,
    size: Cell<Vec2<Span>>,
}

impl Label {
    #[allow(dead_code)]
    pub fn new() -> Rc<Self> {
        Rc::new(Self {
            text: RefCell::new(None),
            col: Cell::new(Rgba::new(0.0, 0.0, 0.0, 1.0)),
            size: Cell::new(Span::px(16, 16)),
        })
    }

    #[allow(dead_code)]
    pub fn with_text(self: Rc<Self>, text: String) -> Rc<Self> {
        *self.text.borrow_mut() = Some(text);
        self
    }

    #[allow(dead_code)]
    pub fn with_color(self: Rc<Self>, col: Rgba<f32>) -> Rc<Self> {
        self.col.set(col);
        self
    }

    #[allow(dead_code)]
    pub fn with_size(self: Rc<Self>, size: Vec2<Span>) -> Rc<Self> {
        self.size.set(size);
        self
    }

    #[allow(dead_code)]
    pub fn get_text(&self) -> Ref<Option<String>> { self.text.borrow() }
    #[allow(dead_code)]
    pub fn set_text(&self, text: String) { *self.text.borrow_mut() = Some(text); }

    #[allow(dead_code)]
    pub fn get_color(&self) -> Rgba<f32> { self.col.get() }
    #[allow(dead_code)]
    pub fn set_color(&self, col: Rgba<f32>) { self.col.set(col); }

    #[allow(dead_code)]
    pub fn get_size(&self) -> Vec2<Span> { self.size.get() }
    #[allow(dead_code)]
    pub fn set_size(&self, size: Vec2<Span>) { self.size.set(size); }

    #[allow(dead_code)]
    pub fn clone_all(&self) -> Rc<Self> { Rc::new(self.clone()) }
}

impl Element for Label {
    fn deep_clone(&self) -> Rc<dyn Element> { self.clone_all() }

    fn render(&self, renderer: &mut Renderer, rescache: &mut ResCache, bounds: (Vec2<f32>, Vec2<f32>)) {
        if let Some(text) = self.text.borrow().as_ref() {
            let res = renderer.get_view_resolution().map(|e| e as f32);
            let sz = self.size.get().map(|e| e.rel) * res.map(|e| e as f32) + self.size.get().map(|e| e.px as f32);
            draw_text(renderer, rescache, text, bounds.0, sz, self.col.get());
        }
    }
}
