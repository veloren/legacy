// Library
use vek::*;

// Local
use super::{Ui, WinBox};

#[test]
fn test_winbox() {
    Ui::new(WinBox::new()
        .with_background(Rgba::one())
    ); // .render();
}
