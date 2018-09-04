// Library
use coord::prelude::*;

// Project
use region::physics;

// Local
use Client;
use ClientStatus;
use Payloads;
use CHUNK_SIZE;

impl<P: Payloads> Client<P> {
    pub(crate) fn tick(&self, dt: f32) -> bool {
        self.update_chunks();
        let entities = self.entities.read();

        // Physics tick
        {
            // Take the physics lock to sync client and frontend updates
            let _ = self.take_phys_lock();
            physics::tick(entities.iter(), &self.chunk_mgr, vec3!(CHUNK_SIZE), dt);
        }

        self.update_server();

        *self.time.write() += dt as f64;

        *self.status() != ClientStatus::Disconnected
    }

    pub(crate) fn tick2(&self, dt: f32) -> bool {
        // Remove chunks that are too far from the player
        // TODO: Could be more efficient (maybe? careful: deadlocks)
        let pers = self.chunk_mgr().persistence();
        pers.offload();

        //generate missing Payloads
        self.lazy_recreate_payload();

        *self.status() != ClientStatus::Disconnected
    }
}
