// Modules
pub mod hbox;
pub mod label;
pub mod rect;
pub mod vbox;
pub mod winbox;

// Rexports
pub use self::{hbox::HBox, label::Label, rect::Rect, vbox::VBox, winbox::WinBox};

// Standard
use std::{cell::RefCell, rc::Rc};

// Local
use super::*;

pub trait Element: 'static {
    fn deep_clone(&self) -> Rc<dyn Element>;
    fn render(&self, renderer: &mut Renderer, rescache: &mut ResCache, bounds: (Vec2<f32>, Vec2<f32>));
}
