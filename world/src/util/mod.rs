// Standard
use std::ops::{Add, Sub, Mul, Div};

pub trait GenFn<I, O> {
    fn eval(&self, i: I) -> O;
}

// ConstantFn

pub struct ConstantFn<V> {
    v: V,
}

impl<V> ConstantFn<V> {
    pub fn new(v: V) -> Self {
        Self { v }
    }
}

impl<I, V: Clone> GenFn<I, V> for ConstantFn<V> {
    fn eval(&self, i: I) -> V {
        self.v.clone()
    }
}

// Impl on common types

impl<T: Clone, I> GenFn<I, Self> for T { fn eval(&self, i: I) -> Self { self.clone() } }

// SquareFn

pub struct SquareFn<F> {
    f: F,
}

impl<F> SquareFn<F> {
    pub fn new(f: F) -> Self {
        Self { f }
    }
}

impl<I, O: Clone, F> GenFn<I, O> for SquareFn<F> where O: Mul<Output=O>, F: GenFn<I, O> {
    fn eval(&self, i: I) -> O {
        let o = self.f.eval(i);
        o.clone() * o
    }
}

// AddFn

pub struct AddFn<F, G> {
    f: F,
    g: G,
}

impl<F, G> AddFn<F, G> {
    pub fn new(f: F, g: G) -> Self {
        Self { f, g }
    }
}

impl<I: Clone, O, F, G> GenFn<I, O> for AddFn<F, G> where O: Add<Output=O>, F: GenFn<I, O>, G: GenFn<I, O> {
    fn eval(&self, i: I) -> O {
        self.f.eval(i.clone()) + self.g.eval(i)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vek::*;

    #[test]
    fn test_genfn_common_types() {
        let _ = SquareFn::new(0u8);
        let _ = SquareFn::new(0u16);
        let _ = SquareFn::new(0u32);
        let _ = SquareFn::new(0u64);

        let _ = SquareFn::new(0i8);
        let _ = SquareFn::new(0i16);
        let _ = SquareFn::new(0i32);
        let _ = SquareFn::new(0i64);

        let _ = SquareFn::new(0.0f32);
        let _ = SquareFn::new(0.0f64);

        let _ = SquareFn::new(true);

        let _ = SquareFn::new(Vec3::broadcast(0u8));
        let _ = SquareFn::new(Vec3::broadcast(0u16));
        let _ = SquareFn::new(Vec3::broadcast(0u32));
        let _ = SquareFn::new(Vec3::broadcast(0u64));

        let _ = SquareFn::new(Vec3::broadcast(0i8));
        let _ = SquareFn::new(Vec3::broadcast(0i16));
        let _ = SquareFn::new(Vec3::broadcast(0i32));
        let _ = SquareFn::new(Vec3::broadcast(0i64));

        let _ = SquareFn::new(Vec3::broadcast(0.0f32));
        let _ = SquareFn::new(Vec3::broadcast(0.0f64));

        let _ = SquareFn::new(Vec3::broadcast(true));
    }

    #[test]
    fn test_constantfn() {
        let x = ConstantFn::new(5);
        assert_eq!(x.eval(0), 5);
    }

    #[test]
    fn test_squarefn() {
        let x = SquareFn::new(ConstantFn::new(5));
        assert_eq!(x.eval(0), 25);
    }

    #[test]
    fn test_addfn() {
        let x = AddFn::new(ConstantFn::new(3), ConstantFn::new(7));
        assert_eq!(x.eval(0), 10);
    }
}
