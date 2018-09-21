// Library
use vek::*;

// Local
use renderer::Renderer;
use super::Element;
use super::ResCache;
use super::primitive::draw_rectangle;

pub struct WinBoxChild {
    anchor: Vec2<f32>,
    offset: Vec2<f32>,
    size: Vec2<f32>,
    element: Box<dyn Element>,
}

pub struct WinBox {
    col: Rgba<f32>,
    children: Vec<WinBoxChild>,
}

impl WinBox {
    pub fn new() -> WinBox {
        WinBox {
            col: Rgba::zero(),
            children: Vec::new(),
        }
    }

    pub fn with_color(mut self, col: Rgba<f32>) -> Self {
        self.col = col;
        self
    }

    pub fn with_child_at<E: Element>(mut self, anchor: Vec2<f32>, offset: Vec2<f32>, size: Vec2<f32>, element: E) -> Self {
        self.children.push(WinBoxChild {
            anchor,
            offset,
            size,
            element: Box::new(element),
        });
        self
    }
}

impl Element for WinBox {
    fn deep_clone(&self) -> Box<dyn Element> {
        Box::new(self.clone())
    }

    fn render(&self, renderer: &mut Renderer, rescache: &mut ResCache, bounds: (Vec2<f32>, Vec2<f32>)) {
        draw_rectangle(renderer, rescache, bounds.0, bounds.1, self.col);

        for WinBoxChild { anchor, offset, size, element } in self.children.iter() {
            element.render(renderer, rescache, (
                bounds.0 + anchor * bounds.1 - offset,
                *size,
            ));
        }
    }
}

impl Clone for WinBox {
    fn clone(&self) -> WinBox {
        WinBox {
            col: self.col,
            children: self.children.iter().map(|c| WinBoxChild {
                anchor: c.anchor,
                offset: c.offset,
                size: c.size,
                element: c.element.deep_clone(),
            }).collect()
        }
    }
}
