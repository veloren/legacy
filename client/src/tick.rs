// Standard
use std::{thread, time::Duration};

// Project
use common::manager::Manager;
use region::physics;

// Local
use Client;
use ClientStatus;
use Payloads;
use CHUNK_SIZE;

impl<P: Payloads> Client<P> {
    pub(crate) fn tick(&self, dt: f32, mgr: &mut Manager<Self>) -> bool {
        self.update_chunks(mgr);
        let entities = self.entities.read();

        // Physics tick
        {
            // Take the physics lock to sync client and frontend updates
            let _ = self.take_phys_lock();
            physics::tick(entities.iter(), &self.chunk_mgr, CHUNK_SIZE, dt);
        }

        self.update_server();

        *self.time.write() += dt as f64;

        thread::sleep(Duration::from_millis(40));

        *self.status() != ClientStatus::Disconnected
    }
}
