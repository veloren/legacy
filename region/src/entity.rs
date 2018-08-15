// Local

// Library
use coord::prelude::*;

pub struct Entity<P: Send + Sync + 'static> {
    pos: Vec3f, //middle x,y of the figure, z pos is on the ground
    vel: Vec3f,
    ctrl_acc: Vec3f,
    look_dir: Vec2f,
    payload: Option<P>,
}

impl<P: Send + Sync + 'static> Entity<P> {
    pub fn new(pos: Vec3f, vel: Vec3f, ctrl_acc: Vec3f, look_dir: Vec2f) -> Entity<P> {
        Entity {
            pos,
            vel,
            ctrl_acc, //entity triest to move in this directory (maybe should be made a acceleration in future versions with correct netwon movement)
            look_dir,
            payload: None,
        }
    }

    pub fn pos(&self) -> &Vec3f { &self.pos }

    pub fn vel(&self) -> &Vec3f { &self.vel }

    pub fn ctrl_acc(&self) -> &Vec3f { &self.ctrl_acc }

    pub fn look_dir(&self) -> &Vec2f { &self.look_dir }

    pub fn pos_mut(&mut self) -> &mut Vec3f { &mut self.pos }

    pub fn vel_mut(&mut self) -> &mut Vec3f { &mut self.vel }

    pub fn ctrl_acc_mut(&mut self) -> &mut Vec3f { &mut self.ctrl_acc }

    pub fn look_dir_mut(&mut self) -> &mut Vec2f { &mut self.look_dir }

    pub fn payload(&self) -> &Option<P> { &self.payload }
    pub fn payload_mut(&mut self) -> &mut Option<P> { &mut self.payload }
}
