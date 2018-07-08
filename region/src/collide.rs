// Local
use {Volume, Voxel};

// Library
use coord::prelude::*;

pub trait VolCollider {
    fn is_solid_at(&self, pos: Vec3<f32>) -> bool;
}

impl<V: Volume> VolCollider for V {
    fn is_solid_at(&self, pos: Vec3<f32>) -> bool {
        self.at(pos.floor().map(|e| e as i64))
            .map(|v| v.is_solid())
            .unwrap_or(false)
    }
}

#[derive(Copy, Clone)]
pub struct AABB {
    p0: Vec3<f32>,
    p1: Vec3<f32>,
}

impl AABB {
    pub fn new(p0: Vec3<f32>, p1: Vec3<f32>) -> AABB {
        AABB { p0, p1 }
    }

    pub fn size(&self) -> Vec3<f32> {
        self.p1 - self.p0
    }

    pub fn collides_with<V: VolCollider>(&self, vol: &V) -> bool {
        let size = self.size().map(|c| c.abs());
        let mut pos = vec3!(0.0, 0.0, 0.0);

        let low_p = vec3!(
            self.p0.x.min(self.p1.x),
            self.p0.y.min(self.p1.y),
            self.p0.z.min(self.p1.z)
        );

        // This logic is horribly long, but works.
        // Yes, the repeated tests are there for a reason.
        // No, it's probably not as slow as you think.
        while pos.x < size.x {
            pos.y = 0.0;
            while pos.y < size.y {
                pos.z = 0.0;
                while pos.z < size.z {
                    if vol.is_solid_at(low_p + pos) {
                        return true;
                    }
                    pos.z = (pos.z + size.z.min(0.2)).min(size.z);
                }
                pos.y = (pos.y + size.y.min(0.2)).min(size.y);
            }
            pos.x = (pos.x + size.x.min(0.2)).min(size.x);
        }
        false
    }

    pub fn shift_by(&mut self, dpos: Vec3<f32>) -> AABB {
        AABB {
            p0: self.p0 + dpos,
            p1: self.p1 + dpos,
        }
    }

    pub fn resolve_with<V: VolCollider>(&self, vol: &V, dpos: Vec3<f32>) -> Vec3<f32> {
        let units = [
            vec3!(0.0, 0.0, 1.0),
            vec3!(0.0, 1.0, 0.0),
            vec3!(1.0, 0.0, 0.0)
        ];

        let dfactor = 0.1;

        let incr = dpos.norm() * dfactor;
        let mut aabb = *self;
        for _ in 0..(dpos.length() / dfactor) as usize {
            for i in 0..3 {
                let tmp = aabb.shift_by(incr * units[i]);
                if !tmp.collides_with(vol) {
                    aabb = tmp;
                }
            }
        }

        aabb.p0 - self.p0
    }
}
