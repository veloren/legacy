// Modules
pub mod button;
pub mod hbox;
pub mod label;
pub mod rect;
pub mod vbox;
pub mod winbox;

// Rexports
pub use self::{button::Button, hbox::HBox, label::Label, rect::Rect, vbox::VBox, winbox::WinBox};

// Standard
use std::{cell::RefCell, rc::Rc};

// Library
use vek::*;

// Local
use super::*;
use window::Event;

// Utility aliases
type Bounds = (Vec2<f32>, Vec2<f32>);

pub trait Element: 'static {
    fn deep_clone(&self) -> Rc<dyn Element>;
    fn render(&self, renderer: &mut Renderer, rescache: &mut ResCache, bounds: Bounds);
    fn handle_event(&self, _event: &Event, _scr_res: Vec2<f32>, _bounds: Bounds) -> bool { false }
}
