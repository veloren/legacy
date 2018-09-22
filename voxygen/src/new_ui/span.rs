// Standard
use std::ops::*;

// Library
use vek::*;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Span {
    pub rel: f32,
    pub px: i32,
}

impl Span {
    pub fn zero() -> Vec2<Self> {
        Self::rel(0.0, 0.0)
    }

    pub fn full() -> Vec2<Self> {
        Self::rel(1.0, 1.0)
    }

    pub fn center() -> Vec2<Self> {
        Self::rel(0.5, 0.5)
    }

    pub fn left() -> Vec2<Self> {
        Self::rel(0.0, 0.5)
    }

    pub fn right() -> Vec2<Self> {
        Self::rel(1.0, 0.5)
    }

    pub fn top() -> Vec2<Self> {
        Self::rel(0.5, 0.0)
    }

    pub fn bottom() -> Vec2<Self> {
        Self::rel(0.5, 1.0)
    }

    pub fn top_left() -> Vec2<Self> {
        Self::rel(0.0, 0.0)
    }

    pub fn top_right() -> Vec2<Self> {
        Self::rel(1.0, 0.0)
    }

    pub fn bottom_left() -> Vec2<Self> {
        Self::rel(0.0, 1.0)
    }

    pub fn bottom_right() -> Vec2<Self> {
        Self::rel(1.0, 1.0)
    }

    pub fn rel(x: f32, y: f32) -> Vec2<Self> {
        Self::rel_and_px(x, y, 0, 0)
    }

    pub fn px(x: i32, y: i32) -> Vec2<Self> {
        Self::rel_and_px(0.0, 0.0, x, y)
    }

    pub fn rel_and_px(rx: f32, ry: f32, px: i32, py: i32) -> Vec2<Self> {
        Vec2::new(
            Span { rel: rx, px: px },
            Span { rel: ry, px: py },
        )
    }
}

impl Add for Span {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self {
            rel: self.rel + rhs.rel,
            px: self.px + rhs.px,
        }
    }
}

impl Sub for Span {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self {
            rel: self.rel - rhs.rel,
            px: self.px - rhs.px,
        }
    }
}

impl From<f32> for Span {
    fn from(r: f32) -> Self {
        Self { rel: r, px: 0 }
    }
}

impl From<i32> for Span {
    fn from(p: i32) -> Self {
        Self { rel: 0.0, px: p }
    }
}
