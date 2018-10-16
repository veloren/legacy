// Standard
use std::ops::{Add, Sub, Mul, Div};
use std::marker::PhantomData;

// Library
use num_traits::float::Float;

pub trait GenFn<I, O>: Sized {
    fn eval(&self, i: I) -> O;

    fn map<M, MO>(self, m: M) -> MapFn<I, Self, O, M, MO> where M: Fn(O) -> MO {
        MapFn::new(self, m)
    }
}

pub trait FloatFn<I, O: Float>: GenFn<I, O> {
    fn powf(self, pow: O) -> PowfFn<Self, O> {
        PowfFn::new(self, pow)
    }
}

// ConstantFn

#[allow(dead_code)]
pub struct ConstantFn<O> {
    o: O,
}

#[allow(dead_code)]
impl<O> ConstantFn<O> {
    pub fn new(o: O) -> Self {
        Self { o }
    }
}

impl<I, O> GenFn<I, O> for ConstantFn<O> where O: Clone {
    fn eval(&self, _: I) -> O {
        self.o.clone()
    }
}

impl<I, O> FloatFn<I, O> for ConstantFn<O> where O: Clone + Float {}

// Impl on common types

impl<I, T> GenFn<I, Self> for T where T: Clone { fn eval(&self, _: I) -> Self { self.clone() } }
impl<I, T> FloatFn<I, Self> for T where T: Clone + Float + GenFn<I, Self> {}

// PowFn

#[allow(dead_code)]
pub struct PowfFn<F, O> {
    f: F,
    pow: O,
}

#[allow(dead_code)]
impl<F, O> PowfFn<F, O> {
    pub fn new(f: F, pow: O) -> Self {
        Self { f, pow }
    }
}

impl<I, F, O> GenFn<I, O> for PowfFn<F, O> where F: FloatFn<I, O>, O: Clone + Float {
    fn eval(&self, i: I) -> O {
        self.f.eval(i).powf(self.pow.clone())
    }
}

// AddFn

#[allow(dead_code)]
pub struct AddFn<F, G> {
    f: F,
    g: G,
}

#[allow(dead_code)]
impl<F, G> AddFn<F, G> {
    pub fn new(f: F, g: G) -> Self {
        Self { f, g }
    }
}

impl<I: Clone, F, G, O> GenFn<I, O> for AddFn<F, G> where F: GenFn<I, O>, G: GenFn<I, O>, O: Add<Output=O> {
    fn eval(&self, i: I) -> O {
        self.f.eval(i.clone()) + self.g.eval(i)
    }
}

// MapFn

#[allow(dead_code)]
pub struct MapFn<I, F, F2M, M, O> {
    f: F,
    m: M,
    _phantom: PhantomData<(I, F2M, O)>,
}

#[allow(dead_code)]
impl<I, F, F2M, M, O> MapFn<I, F, F2M, M, O> {
    pub fn new(f: F, m: M) -> Self {
        Self { f, m, _phantom: PhantomData }
    }
}

impl<I, F, F2M, M, O> GenFn<I, O> for MapFn<I, F, F2M, M, O> where F: GenFn<I, F2M>, M: Fn(F2M) -> O {
    fn eval(&self, i: I) -> O {
        self.m.call((self.f.eval(i), ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vek::*;

    #[test]
    fn test_genfn_common_types() {
        let _ = ConstantFn::new(0u8);
        let _ = ConstantFn::new(0u16);
        let _ = ConstantFn::new(0u32);
        let _ = ConstantFn::new(0u64);

        let _ = ConstantFn::new(0i8);
        let _ = ConstantFn::new(0i16);
        let _ = ConstantFn::new(0i32);
        let _ = ConstantFn::new(0i64);

        let _ = ConstantFn::new(0.0f32);
        let _ = ConstantFn::new(0.0f64);

        let _ = ConstantFn::new(true);

        let _ = ConstantFn::new(Vec3::broadcast(0u8));
        let _ = ConstantFn::new(Vec3::broadcast(0u16));
        let _ = ConstantFn::new(Vec3::broadcast(0u32));
        let _ = ConstantFn::new(Vec3::broadcast(0u64));

        let _ = ConstantFn::new(Vec3::broadcast(0i8));
        let _ = ConstantFn::new(Vec3::broadcast(0i16));
        let _ = ConstantFn::new(Vec3::broadcast(0i32));
        let _ = ConstantFn::new(Vec3::broadcast(0i64));

        let _ = ConstantFn::new(Vec3::broadcast(0.0f32));
        let _ = ConstantFn::new(Vec3::broadcast(0.0f64));

        let _ = ConstantFn::new(Vec3::broadcast(true));
    }

    #[test]
    fn test_constantfn() {
        let x = ConstantFn::new(5);
        assert_eq!(x.eval(0), 5);
    }

    #[test]
    fn test_powfn() {
        let x = PowfFn::new(ConstantFn::new(5.0), 2.0);
        assert_eq!(x.eval(0), 25.0);
    }

    #[test]
    fn test_addfn() {
        let x = AddFn::new(ConstantFn::new(3), ConstantFn::new(7));
        assert_eq!(x.eval(0), 10);
    }

    #[test]
    fn test_mapfn() {
        let x = MapFn::new(ConstantFn::new(3), |v| v * 3);
        assert_eq!(x.eval(0), 9);
    }

    #[test]
    fn test_map() {
        let x = ConstantFn::new(3).map(|v| v * 3);
        assert_eq!(x.eval(0), 9);
    }
}
