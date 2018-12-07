pub mod collision;
pub mod movement;
pub mod physics;
#[cfg(test)]
mod tests;

/*
Physics is seperated into multiple layers.
Lowest layer is collision.rs, it handles collisions between 1 stationary Primitiv and one moving Primitive.
It can return where and when they will collide.
Based on this we have movement.rs, it contains all kind of abstracted general valid movement algorithm.
The physics.rs code contains all physics code which is applied to entities, it has specific branches for entities,
based on their speed, movement prediction, friction, block hopping and depends on chunk_mgr and entities
*/
