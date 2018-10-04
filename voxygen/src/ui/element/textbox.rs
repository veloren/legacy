// Standard
use std::{
    cell::{Cell, Ref, RefCell},
    rc::Rc,
};

// Library
use vek::*;

// Local
use super::{
    primitive::{draw_rectangle, draw_text},
    Bounds, Element, Event, ResCache, Span,
};
use renderer::Renderer;

#[allow(dead_code)]
#[derive(Clone)]
pub struct TextBox {
    text: RefCell<String>,
    col: Cell<Rgba<f32>>,
    bg_col: Cell<Rgba<f32>>,
    margin: Cell<Vec2<Span>>,
    size: Cell<Vec2<Span>>,
    return_fn: RefCell<Option<Rc<dyn Fn(&TextBox, &str) + 'static>>>,
}

impl TextBox {
    #[allow(dead_code)]
    pub fn new() -> Rc<Self> {
        Rc::new(Self {
            text: RefCell::new("".to_string()),
            col: Cell::new(Rgba::new(0.0, 0.0, 0.0, 1.0)),
            bg_col: Cell::new(Rgba::new(1.0, 1.0, 1.0, 1.0)),
            margin: Cell::new(Span::zero()),
            size: Cell::new(Span::px(16, 16)),
            return_fn: RefCell::new(None),
        })
    }

    #[allow(dead_code)]
    pub fn with_text(self: Rc<Self>, text: String) -> Rc<Self> {
        *self.text.borrow_mut() = text;
        self
    }

    #[allow(dead_code)]
    pub fn with_color(self: Rc<Self>, col: Rgba<f32>) -> Rc<Self> {
        self.col.set(col);
        self
    }

    #[allow(dead_code)]
    pub fn with_background_color(self: Rc<Self>, col: Rgba<f32>) -> Rc<Self> {
        self.bg_col.set(col);
        self
    }

    #[allow(dead_code)]
    pub fn with_margin(self: Rc<Self>, margin: Vec2<Span>) -> Rc<Self> {
        self.margin.set(margin);
        self
    }

    #[allow(dead_code)]
    pub fn with_return_fn<F: Fn(&Self, &str) + 'static>(self: Rc<Self>, f: F) -> Rc<Self> {
        *self.return_fn.borrow_mut() = Some(Rc::new(f));
        self
    }

    #[allow(dead_code)]
    pub fn with_size(self: Rc<Self>, size: Vec2<Span>) -> Rc<Self> {
        self.size.set(size);
        self
    }

    #[allow(dead_code)]
    pub fn get_text(&self) -> Ref<String> { self.text.borrow() }
    #[allow(dead_code)]
    pub fn set_text(&self, text: String) { *self.text.borrow_mut() = text; }

    #[allow(dead_code)]
    pub fn get_color(&self) -> Rgba<f32> { self.col.get() }
    #[allow(dead_code)]
    pub fn set_color(&self, col: Rgba<f32>) { self.col.set(col); }

    #[allow(dead_code)]
    pub fn get_background_color(&self) -> Rgba<f32> { self.bg_col.get() }
    #[allow(dead_code)]
    pub fn set_background_color(&self, bg_col: Rgba<f32>) { self.bg_col.set(bg_col); }

    #[allow(dead_code)]
    pub fn set_return_fn<F: Fn(&Self, &str) + 'static>(&self, f: F) { *self.return_fn.borrow_mut() = Some(Rc::new(f)); }

    #[allow(dead_code)]
    pub fn get_size(&self) -> Vec2<Span> { self.size.get() }
    #[allow(dead_code)]
    pub fn set_size(&self, size: Vec2<Span>) { self.size.set(size); }

    #[allow(dead_code)]
    pub fn clone_all(&self) -> Rc<Self> { Rc::new(self.clone()) }
}

impl Element for TextBox {
    fn deep_clone(&self) -> Rc<dyn Element> { self.clone_all() }

    fn render(&self, renderer: &mut Renderer, rescache: &mut ResCache, bounds: Bounds) {
        draw_rectangle(renderer, rescache, bounds.0, bounds.1, self.bg_col.get());

        let scr_res = renderer.get_view_resolution().map(|e| e as f32);
        let margin_rel = self.margin.get().map(|e| e.rel) * bounds.1 + self.margin.get().map(|e| e.px as f32) / scr_res;

        let child_bounds = (bounds.0 + margin_rel, bounds.1 - margin_rel * 2.0);
        let sz = self.size.get().map(|e| e.rel) * scr_res.map(|e| e as f32) + self.size.get().map(|e| e.px as f32);
        draw_text(
            renderer,
            rescache,
            &self.text.borrow(),
            child_bounds.0,
            sz,
            self.col.get(),
        );
    }

    fn handle_event(&self, event: &Event, _scr_res: Vec2<f32>, _bounds: Bounds) -> bool {
        match event {
            Event::Character { ch } => {
                match ch {
                    '\n' | '\r' => {
                        let mut text = self.text.borrow_mut();
                        self.return_fn.borrow_mut().as_mut().map(|f| (*f)(self, &text));
                        text.clear();
                    },
                    '\x08' => {
                        self.text.borrow_mut().pop();
                    },
                    c => {
                        self.text.borrow_mut().push(*c);
                    },
                }
                true
            },
            Event::KeyboardInput { .. } => true,
            _ => false,
        }
    }
}
