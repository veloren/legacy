// Standard
use std::time::Duration;

// Project
use common::{physics::physics, util::manager::Manager};

// Local
use Client;
use ClientStatus;
use Payloads;

impl<P: Payloads> Client<P> {
    pub(crate) fn tick(&self, dt: Duration, _mgr: &mut Manager<Self>) -> bool {
        let entities = self.entities.read();

        // Physics tick
        {
            // Take the physics lock to sync client and frontend updates
            let _ = self.take_phys_lock();
            //physics::tick(entities.iter(), &self.chunk_mgr, dt);

            // TODO: Fix this
            if let Some(entity) = self.player_entity() {
                let e = [entity.clone()];
                physics::tick(e.into_iter(), &self.chunk_mgr, Vec3::from_slice(&CHUNK_SIZE), dt);
            }
        }

        self.update_server();

        *self.status() != ClientStatus::Disconnected
    }

    pub(crate) fn manage_chunks(&self, mgr: &mut Manager<Self>) -> bool {
        self.maintain_chunks(mgr);
        *self.status() != ClientStatus::Disconnected
    }

    pub(crate) fn debug(&self, _mgr: &mut Manager<Self>) -> bool {
        self.chunk_mgr().debug();
        *self.status() != ClientStatus::Disconnected
    }
}
