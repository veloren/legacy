// Library
use vek::*;

pub struct Entity<P: Send + Sync + 'static> {
    pos: Vec3<f32>, //middle x,y of the figure, z pos is on the ground
    vel: Vec3<f32>,
    ctrl_acc: Vec3<f32>,
    look_dir: Vec2<f32>,
    payload: Option<P>,
}

impl<P: Send + Sync + 'static> Entity<P> {
    pub fn new(pos: Vec3<f32>, vel: Vec3<f32>, ctrl_acc: Vec3<f32>, look_dir: Vec2<f32>) -> Entity<P> {
        Entity {
            pos,
            vel,
            ctrl_acc, //entity triest to move in this directory (maybe should be made a acceleration in future versions with correct netwon movement)
            look_dir,
            payload: None,
        }
    }

    pub fn pos(&self) -> &Vec3<f32> { &self.pos }

    pub fn vel(&self) -> &Vec3<f32> { &self.vel }

    pub fn ctrl_acc(&self) -> &Vec3<f32> { &self.ctrl_acc }

    pub fn look_dir(&self) -> &Vec2<f32> { &self.look_dir }

    pub fn pos_mut(&mut self) -> &mut Vec3<f32> { &mut self.pos }

    pub fn vel_mut(&mut self) -> &mut Vec3<f32> { &mut self.vel }

    pub fn ctrl_acc_mut(&mut self) -> &mut Vec3<f32> { &mut self.ctrl_acc }

    pub fn look_dir_mut(&mut self) -> &mut Vec2<f32> { &mut self.look_dir }

    pub fn payload(&self) -> &Option<P> { &self.payload }
    pub fn payload_mut(&mut self) -> &mut Option<P> { &mut self.payload }
}
