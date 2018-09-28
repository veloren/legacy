// Standard
use std::{cell::{Cell, RefCell}, rc::Rc};

// Library
use vek::*;
use glutin::{MouseButton, ElementState};

// Local
use super::{
    primitive::{draw_text, draw_rectangle},
    Element,
    ResCache,
    Span,
    Event,
    Bounds,
};
use renderer::Renderer;

#[allow(dead_code)]
pub struct Button {
    col: Cell<Rgba<f32>>,
    padding: Cell<Vec2<Span>>,
    mouseover: Cell<bool>,
    click_fn: RefCell<Option<Rc<dyn Fn(&Button) + 'static>>>,
    child: RefCell<Option<Rc<dyn Element>>>,
}

impl Button {
    #[allow(dead_code)]
    pub fn new() -> Rc<Self> {
        Rc::new(Self {
            col: Cell::new(Rgba::one()),
            padding: Cell::new(Span::zero()),
            mouseover: Cell::new(false),
            click_fn: RefCell::new(None),
            child: RefCell::new(None),
        })
    }

    #[allow(dead_code)]
    pub fn with_color(self: Rc<Self>, col: Rgba<f32>) -> Rc<Self> {
        self.col.set(col);
        self
    }

    #[allow(dead_code)]
    pub fn with_padding(self: Rc<Self>, padding: Vec2<Span>) -> Rc<Self> {
        self.padding.set(padding);
        self
    }

    #[allow(dead_code)]
    pub fn with_click_fn<F: Fn(&Self) + 'static>(self: Rc<Self>, f: F) -> Rc<Self> {
        *self.click_fn.borrow_mut() = Some(Rc::new(f));
        self
    }

    #[allow(dead_code)]
    pub fn with_child<E: Element>(self: Rc<Self>, child: Rc<E>) -> Rc<Self> {
        *self.child.borrow_mut() = Some(child);
        self
    }

    #[allow(dead_code)]
    pub fn get_color(&self) -> Rgba<f32> { self.col.get() }
    #[allow(dead_code)]
    pub fn set_color(&self, col: Rgba<f32>) { self.col.set(col); }

    #[allow(dead_code)]
    pub fn get_padding(&self) -> Vec2<Span> { self.padding.get() }
    #[allow(dead_code)]
    pub fn set_padding(&self, padding: Vec2<Span>) { self.padding.set(padding); }

    #[allow(dead_code)]
    pub fn get_child(&self) -> Option<Rc<dyn Element>> {
        self.child.borrow().as_ref().map(|c| c.clone())
    }
    #[allow(dead_code)]
    pub fn set_child<E: Element>(&self, child: Rc<E>) -> Rc<E> {
        *self.child.borrow_mut() = Some(child.clone());
        child
    }

    #[allow(dead_code)]
    pub fn clone_all(&self) -> Rc<Self> { Rc::new(self.clone()) }

    fn bounds_for_child(&self, scr_res: Vec2<f32>, bounds: Bounds) -> Bounds {
        let padding_rel = self.padding.get().map(|e| e.rel) * bounds.1 * scr_res + self.padding.get().map(|e| e.px as f32) / scr_res;
        (bounds.0 + padding_rel, bounds.1 - padding_rel * 2.0)
    }
}

impl Element for Button {
    fn deep_clone(&self) -> Rc<dyn Element> { self.clone_all() }

    fn render(&self, renderer: &mut Renderer, rescache: &mut ResCache, bounds: Bounds) {
        let scr_res = renderer.get_view_resolution().map(|e| e as f32);

        let child_bounds = self.bounds_for_child(scr_res, bounds);

        draw_rectangle(
            renderer,
            rescache,
            child_bounds.0,
            child_bounds.1,
            self.col.get() - if self.mouseover.get() {
                Rgba::new(0.3, 0.3, 0.3, 0.0)
            } else {
                Rgba::zero()
            },
        );
    }

    fn handle_event(&self, event: &Event, scr_res: Vec2<f32>, bounds: Bounds) -> bool {
        let used = self.child.borrow().as_ref().map(|child| child.handle_event(
            event,
            scr_res,
            self.bounds_for_child(scr_res, bounds),
        )).unwrap_or(false);

        match event {
            Event::CursorPosition { x, y } => {
                let cursor = Vec2::new(*x as f32, *y as f32) / scr_res;
                if
                    cursor.x > bounds.0.x &&
                    cursor.y > bounds.0.y &&
                    cursor.x < bounds.0.x + bounds.1.x &&
                    cursor.y < bounds.0.y + bounds.1.y
                {
                    self.mouseover.set(true);
                } else {
                    self.mouseover.set(false);
                }
            },
            Event::MouseButton { state, button } => {
                if self.mouseover.get() && *button == MouseButton::Left && *state == ElementState::Pressed {
                    self.click_fn.borrow_mut().as_mut().map(|f| (*f)(self));
                }
            },
            _ => {},
        }

        used
    }
}

impl Clone for Button {
    fn clone(&self) -> Self {
        Self {
            col: self.col.clone(),
            padding: self.padding.clone(),
            mouseover: self.mouseover.clone(),
            click_fn: RefCell::new(self.click_fn.borrow().as_ref().map(|c| c.clone())),
            child: RefCell::new(self.child.borrow().as_ref().map(|c| c.deep_clone())),
        }
    }
}
