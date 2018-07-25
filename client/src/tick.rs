// Library
use coord::prelude::*;

// Project
use region::{VolState, physics};

// Local
use {Client, Payloads, ClientStatus, CHUNK_SIZE};

impl<P: Payloads> Client<P> {

    /*
    pub(crate) fn update_physics(&self, dt: f32) {
        // Apply gravity to the play if they are both within a loaded chunk and have ground beneath their feet
        // TODO: We should be able to make this much smaller
        if let Some(player_entity) = self.player_entity() {
            let mut player_entity = player_entity.write().unwrap();

            let player_chunk = player_entity
                .pos()
                .map(|e| e as i64)
                .div_euc(vec3!([CHUNK_SIZE; 3]));

            // Apply gravity to the player
            if let Some(c) = self.chunk_mgr().at(vec2!(player_chunk.x, player_chunk.y)) {
                if let VolState::Exists(_, _) = *c.read().unwrap() {
                    let _below_feet = *player_entity.pos() - vec3!(0.0, 0.0, -0.1);
                    if player_entity // Get the player's...
                        .get_lower_aabb() // ...bounding box...
                        .shift_by(vec3!(0.0, 0.0, -0.1)) // ...move it a little below the player...
                        .collides_with(self.chunk_mgr()) { // ...and check whether it collides with the ground.
                        if player_entity.jumping() {
                            player_entity.vel_mut().z = 3.0;
                        } else {
                            player_entity.vel_mut().z = 0.0;
                        }
                    } else {
                        player_entity.vel_mut().z -= 0.12; // Apply gravity
                    }
                }
            } else {
                player_entity.vel_mut().z = 0.0;
            }
        }

        // Move all entities, avoiding collisions
        for (_uid, entity) in self.entities_mut().iter_mut() {
            let mut entity = entity.write().unwrap();
            // First, calculate the change in position assuming no external influences
            let mut dpos = (*entity.vel() + *entity.ctrl_vel()) * dt;

            // Resolve collisions with the terrain, altering the change in position accordingly
            dpos = entity.get_upper_aabb().resolve_with(self.chunk_mgr(), dpos);

            // Change the entity's position
            *entity.pos_mut() += dpos;

            // Make the player hop up 1-block steps
            if entity.get_lower_aabb().collides_with(self.chunk_mgr()) {
                entity.pos_mut().z += 0.2;
            }
        }
    }*/

    pub(crate) fn tick(&self, dt: f32) -> bool {
        self.update_chunks();
        physics::tick(&self.entities, &self.chunk_mgr, CHUNK_SIZE, dt);
        self.update_server();

        *self.status() != ClientStatus::Disconnected
    }
}
