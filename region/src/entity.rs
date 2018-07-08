// Local
use collide::AABB;

// Library
use coord::prelude::*;

pub struct Entity {
    pos: Vec3f,
    vel: Vec3f,
    ctrl_vel: Vec3f,
    look_dir: Vec2f,
}

impl Entity {
    pub fn new(pos: Vec3f, vel: Vec3f, ctrl_vel: Vec3f, look_dir: Vec2f) -> Entity {
        Entity {
            pos,
            vel,
            ctrl_vel, //entity triest to move in this directory (maybe should be made a acceleration in future versions with correct netwon movement)
            look_dir,
        }
    }

    pub fn pos(&self) -> &Vec3f {
        &self.pos
    }

    pub fn vel(&self) -> &Vec3f {
        &self.vel
    }

    pub fn ctrl_vel(&self) -> &Vec3f {
        &self.ctrl_vel
    }

    pub fn look_dir(&self) -> &Vec2f {
        &self.look_dir
    }

    pub fn pos_mut(&mut self) -> &mut Vec3f {
        &mut self.pos
    }

    pub fn vel_mut(&mut self) -> &mut Vec3f {
        &mut self.vel
    }

    pub fn ctrl_vel_mut(&mut self) -> &mut Vec3f {
        &mut self.ctrl_vel
    }

    pub fn look_dir_mut(&mut self) -> &mut Vec2f {
        &mut self.look_dir
    }

    pub fn get_lower_aabb(&self) -> AABB {
        AABB::new(
            self.pos + vec3!(-0.45, -0.45, 0.0),
            self.pos + vec3!( 0.45,  0.45, 1.5),
        )
    }

    pub fn get_upper_aabb(&self) -> AABB {
        AABB::new(
            self.pos + vec3!(-0.45, -0.45, 1.5),
            self.pos + vec3!( 0.45,  0.45, 1.8),
        )
    }
}
