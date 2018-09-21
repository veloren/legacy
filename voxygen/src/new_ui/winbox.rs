// Library
use vek::*;

// Local
use renderer::Renderer;
use super::Element;
use super::ResCache;

// WinBox

pub struct WinBoxChild {
    anchor: Vec2<f32>,
    offset: Vec2<f32>,
    element: Box<dyn Element>,
}

pub struct WinBox {
    bg_col: Rgba<f32>,
    children: Vec<WinBoxChild>,
}

impl WinBox {
    pub fn new() -> WinBox {
        WinBox {
            bg_col: Rgba::zero(),
            children: Vec::new(),
        }
    }

    pub fn with_background(mut self, bg_col: Rgba<f32>) -> Self {
        self.bg_col = bg_col;
        self
    }
}

impl Element for WinBox {
    fn deep_clone(&self) -> Box<dyn Element> {
        Box::new(self.clone())
    }

    fn render(&self, renderer: &mut Renderer, res_cache: &mut ResCache, trans: Mat2<f32>) {
        unimplemented!();
    }
}

impl Clone for WinBox {
    fn clone(&self) -> WinBox {
        WinBox {
            bg_col: self.bg_col,
            children: self.children.iter().map(|c| WinBoxChild {
                anchor: c.anchor,
                offset: c.offset,
                element: c.element.deep_clone(),
            }).collect()
        }
    }
}
