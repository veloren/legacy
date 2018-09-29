// Standard
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

// Library
use glutin::{ElementState, MouseButton};
use vek::*;

// Local
use super::{
    primitive::{draw_rectangle, draw_text},
    Bounds, Element, Event, ResCache, Span,
};
use renderer::Renderer;

#[derive(Copy, Clone, PartialEq)]
enum ActiveMode {
    None,
    Hover,
    Click,
}

#[allow(dead_code)]
pub struct Button {
    col: Cell<Rgba<f32>>,
    hover_col: Cell<Rgba<f32>>,
    click_col: Cell<Rgba<f32>>,
    margin: Cell<Vec2<Span>>,
    active_mode: Cell<ActiveMode>,
    click_fn: RefCell<Option<Rc<dyn Fn(&Button) + 'static>>>,
    child: RefCell<Option<Rc<dyn Element>>>,
}

impl Button {
    #[allow(dead_code)]
    pub fn new() -> Rc<Self> {
        Rc::new(Self {
            col: Cell::new(Rgba::one()),
            hover_col: Cell::new(Rgba::one()),
            click_col: Cell::new(Rgba::one()),
            margin: Cell::new(Span::zero()),
            active_mode: Cell::new(ActiveMode::None),
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
    pub fn with_hover_color(self: Rc<Self>, col: Rgba<f32>) -> Rc<Self> {
        self.hover_col.set(col);
        self
    }

    #[allow(dead_code)]
    pub fn with_click_color(self: Rc<Self>, col: Rgba<f32>) -> Rc<Self> {
        self.click_col.set(col);
        self
    }

    #[allow(dead_code)]
    pub fn with_margin(self: Rc<Self>, margin: Vec2<Span>) -> Rc<Self> {
        self.margin.set(margin);
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
    pub fn get_hover_color(&self) -> Rgba<f32> { self.hover_col.get() }
    #[allow(dead_code)]
    pub fn set_hover_color(&self, col: Rgba<f32>) { self.hover_col.set(col); }

    #[allow(dead_code)]
    pub fn get_click_color(&self) -> Rgba<f32> { self.click_col.get() }
    #[allow(dead_code)]
    pub fn set_click_color(&self, col: Rgba<f32>) { self.click_col.set(col); }

    #[allow(dead_code)]
    pub fn get_margin(&self) -> Vec2<Span> { self.margin.get() }
    #[allow(dead_code)]
    pub fn set_margin(&self, margin: Vec2<Span>) { self.margin.set(margin); }

    #[allow(dead_code)]
    pub fn set_click_fn<F: Fn(&Self) + 'static>(&self, f: F) { *self.click_fn.borrow_mut() = Some(Rc::new(f)); }

    #[allow(dead_code)]
    pub fn get_child(&self) -> Option<Rc<dyn Element>> { self.child.borrow().as_ref().map(|c| c.clone()) }
    #[allow(dead_code)]
    pub fn set_child<E: Element>(&self, child: Rc<E>) -> Rc<E> {
        *self.child.borrow_mut() = Some(child.clone());
        child
    }

    #[allow(dead_code)]
    pub fn clone_all(&self) -> Rc<Self> { Rc::new(self.clone()) }

    fn bounds_for_child(&self, scr_res: Vec2<f32>, bounds: Bounds) -> Bounds {
        let margin_rel =
            self.margin.get().map(|e| e.rel) * bounds.1 * scr_res + self.margin.get().map(|e| e.px as f32) / scr_res;
        (bounds.0 + margin_rel, bounds.1 - margin_rel * 2.0)
    }
}

impl Element for Button {
    fn deep_clone(&self) -> Rc<dyn Element> { self.clone_all() }

    fn render(&self, renderer: &mut Renderer, rescache: &mut ResCache, bounds: Bounds) {
        let scr_res = renderer.get_view_resolution().map(|e| e as f32);

        draw_rectangle(
            renderer,
            rescache,
            bounds.0,
            bounds.1,
            match self.active_mode.get() {
                ActiveMode::None => self.col.get(),
                ActiveMode::Hover => self.hover_col.get(),
                ActiveMode::Click => self.click_col.get(),
            },
        );

        let child_bounds = self.bounds_for_child(scr_res, bounds);

        if let Some(child) = self.child.borrow().as_ref() {
            child.render(renderer, rescache, child_bounds);
        }
    }

    fn handle_event(&self, event: &Event, scr_res: Vec2<f32>, bounds: Bounds) -> bool {
        let used = self
            .child
            .borrow()
            .as_ref()
            .map(|child| child.handle_event(event, scr_res, self.bounds_for_child(scr_res, bounds)))
            .unwrap_or(false);

        let used = used | match event {
            Event::CursorPosition { x, y } => {
                let cursor = Vec2::new(*x as f32, *y as f32) / scr_res;
                if cursor.x > bounds.0.x
                    && cursor.y > bounds.0.y
                    && cursor.x < bounds.0.x + bounds.1.x
                    && cursor.y < bounds.0.y + bounds.1.y
                {
                    if (self.active_mode.get() == ActiveMode::None) {
                        self.active_mode.set(ActiveMode::Hover);
                    }
                } else {
                    self.active_mode.set(ActiveMode::None);
                }
                false
            },
            Event::MouseButton { state, button } => {
                if self.active_mode.get() != ActiveMode::None && *button == MouseButton::Left {
                    if *state == ElementState::Pressed {
                        self.active_mode.set(ActiveMode::Click);
                    } else {
                        self.click_fn.borrow_mut().as_mut().map(|f| (*f)(self));
                        self.active_mode.set(ActiveMode::Hover);
                    }
                    true
                } else {
                    false
                }
            },
            _ => false,
        };

        used
    }
}

impl Clone for Button {
    fn clone(&self) -> Self {
        Self {
            col: self.col.clone(),
            hover_col: self.hover_col.clone(),
            click_col: self.click_col.clone(),
            margin: self.margin.clone(),
            active_mode: self.active_mode.clone(),
            click_fn: RefCell::new(self.click_fn.borrow().as_ref().map(|c| c.clone())),
            child: RefCell::new(self.child.borrow().as_ref().map(|c| c.deep_clone())),
        }
    }
}
