// Standard
use std::{thread, time::Duration};

// Library
use vek::*;

// Project
use common::{physics::physics, util::manager::Manager};

// Local
use Client;
use ClientStatus;
use Payloads;
use CHUNK_SIZE;

impl<P: Payloads> Client<P> {
    pub(crate) fn tick(&self, dt: f32, _mgr: &mut Manager<Self>) -> bool {
        let entities = self.entities.read();

        // Physics tick
        {
            // Take the physics lock to sync client and frontend updates
            let _ = self.take_phys_lock();
            physics::tick(entities.iter(), &self.chunk_mgr, Vec3::from_slice(&CHUNK_SIZE), dt);
        }

        self.update_server();

        *self.time.write() += dt as f64;

        thread::sleep(Duration::from_millis(40));

        *self.status() != ClientStatus::Disconnected
    }

    pub(crate) fn manage_chunks(&self, _dt: f32, mgr: &mut Manager<Self>) -> bool {
        self.load_unload_chunks(mgr);
        self.chunk_mgr().persistence().try_cold_offload();
        self.chunk_mgr().persistence().debug();
        thread::sleep(Duration::from_millis(500));
        *self.status() != ClientStatus::Disconnected
    }
}
