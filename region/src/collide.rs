// Local
use collision::Collidable;
use {Volume, Voxel};

// Library
use coord::prelude::*;

pub trait VolCollider {
    fn is_solid_at(&self, pos: Vec3<f32>) -> bool;
    fn get_nearby(&self, pos: Vec3<f32>, radius: Vec3<f32>) -> Vec<Collidable>;
    fn scale(&self) -> Vec3<f32>;
}

impl<V: Volume> VolCollider for V {
    fn is_solid_at(&self, pos: Vec3<f32>) -> bool {
        self.at(pos.floor().map(|e| e as i64))
            .map(|v| v.is_solid())
            .unwrap_or(false)
    }

    fn get_nearby(&self, pos: Vec3<f32>, radius: Vec3<f32>) -> Vec<Collidable> {
        let mut result = Vec::new();
        let area = radius + self.scale();
        let area = vec3!(area.x as i64, area.y as i64, area.z as i64) + vec3!(1,1,1);

        let posi = vec3!(pos.x as i64, pos.y as i64, pos.z as i64);
        let low = posi - area;
        let high = posi + area + vec3!(1,1,1);

        for x in low.z..high.z {
            for y in low.x..high.x {
                for z in low.y..high.y {
                    if self.is_solid_at(vec3!(x as f32,y as f32,z as f32)) {
                        let col = Collidable::new_cuboid(vec3!(x as f32 + 0.5, y as f32 + 0.5, z as f32 + 0.5), vec3!(0.5, 0.5, 0.5));
                        result.push(col);
                    }
                }
            }
        }
        return result;
    }

    fn scale(&self) -> Vec3<f32> {
        self.scale()
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
        let size = self.size();
        let mut pos = vec3!(0.0, 0.0, 0.0);

        // This logic is horribly long, but works.
        // Yes, the repeated tests are there for a reason.
        // No, it's probably not as slow as you think.
        while pos.x < size.x {
            pos.y = 0.0;
            while pos.y < size.y {
                pos.z = 0.0;
                while pos.z < size.z {
                    if vol.is_solid_at(self.p0 + pos) {
                        return true;
                    }
                    pos.z = (pos.z + 0.5).min(size.z);
                    if vol.is_solid_at(self.p0 + pos) {
                        return true;
                    }
                }
                pos.y = (pos.y + 0.5).min(size.y);
                if vol.is_solid_at(self.p0 + pos) {
                    return true;
                }
            }
            pos.x = (pos.x + 0.5).min(size.x);
            if vol.is_solid_at(self.p0 + pos) {
                return true;
            }
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
