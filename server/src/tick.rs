// Standard
use std::time::Duration;

// Local
use Payloads;
use Server;

// Server

impl<P: Payloads> Server<P> {
    pub fn tick_once(&mut self, dt: Duration) {
        self.time += dt.as_millis() as f64 / 1000.0;

        // Sync entities with connected players
        self.sync_players();

        self.world.maintain();
    }
}
