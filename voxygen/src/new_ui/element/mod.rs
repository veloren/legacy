// Modules
pub mod winbox;
pub mod vbox;
pub mod hbox;
pub mod rect;
pub mod label;

// Rexports
pub use self::winbox::WinBox;
pub use self::vbox::VBox;
pub use self::hbox::HBox;
pub use self::rect::Rect;
pub use self::label::Label;

// Standard
use std::{
    rc::Rc,
    cell::RefCell
};

// Local
use super::*;

pub trait Element: 'static {
    fn deep_clone(&self) -> Rc<RefCell<dyn Element>>;
    fn render(&self, renderer: &mut Renderer, rescache: &mut ResCache, bounds: (Vec2<f32>, Vec2<f32>));
}
