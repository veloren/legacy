// Local
use Payloads;
use Server;

use std::time::Duration;

// Server

impl<P: Payloads> Server<P> {
    pub fn tick_once(&mut self, _dt: Duration) {
        // Sync entities with connected players
        self.sync_players();

        self.world.maintain();
    }

    pub fn tick_time(&mut self) {
        // Sync entities with current time
        self.sync_player_time();
    }
}
