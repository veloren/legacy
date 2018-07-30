// Library
use coord::prelude::*;

// Project
use region::{VolState, physics};

// Local
use {Client, Payloads, ClientStatus, CHUNK_SIZE};

impl<P: Payloads> Client<P> {

    pub(crate) fn tick(&self, dt: f32) -> bool {
        self.update_chunks();
        let entities = self.entities.read().unwrap();
        physics::tick(entities.iter(), &self.chunk_mgr, CHUNK_SIZE, dt);
        self.update_server();

        *self.status() != ClientStatus::Disconnected
    }
}
