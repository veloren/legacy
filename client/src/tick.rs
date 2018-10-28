// Standard
use std::{thread, time::Duration};

// Project
use common::{physics::physics, util::manager::Manager};

// Local
use Client;
use ClientStatus;
use Payloads;

impl<P: Payloads> Client<P> {
    pub(crate) fn tick(&self, dt: f32, mgr: &mut Manager<Self>) -> bool {
        let entities = self.entities.read();

        // Physics tick
        {
            // Take the physics lock to sync client and frontend updates
            let _ = self.take_phys_lock();
            physics::tick(entities.iter(), &self.chunk_mgr, dt);
        }

        self.update_server();

        *self.time.write() += dt as f64;

        thread::sleep(Duration::from_millis(40));

        *self.status() != ClientStatus::Disconnected
    }

    pub(crate) fn manage_chunks(&self, dt: f32, mgr: &mut Manager<Self>) -> bool {
        self.maintain_chunks(mgr);

        thread::sleep(Duration::from_millis(500));
        *self.status() != ClientStatus::Disconnected
    }
}
