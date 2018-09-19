// Library
use specs::prelude::*;
use parking_lot::RwLock;

// Project
use common::{
    manager::Manager,
    msg::{SessionKind, ServerMsg},
    Uid,
};

// Local
use Payloads;
use Server;
use net::{Client, DisconnectReason};
use player::Player;

pub trait Api {
    fn disconnect_player(&mut self, player: Entity, reason: DisconnectReason);
    fn broadcast_msg(&self, text: &str);

    fn world(&self) -> &World;
    fn world_mut(&mut self) -> &mut World;
}

impl<P: Payloads> Api for Server<P> {
    fn disconnect_player(&mut self, client: Entity, reason: DisconnectReason) {
        // Stop the postoffice
        if let Some(client) = self.world.read_storage::<Client>().get(client) {
            client.postoffice.stop();
        }

        if let Some(player) = self.world.read_storage::<Player>().get(client) {
            self.broadcast_msg(&format!("[{} disconnected: {}]", player.alias, reason));
            self.payload.on_client_disconnect(self, client, reason);
        }

        self.world.delete_entity(client);
    }

    fn broadcast_msg(&self, text: &str) {
        let clients = self.world.read_storage::<Client>();
        for entity in self.world.entities().join() {
            if let Some(client) = clients.get(entity) {
                client.postoffice.send_one(ServerMsg::ChatMsg { text: text.to_string() });
            }
        }
    }

    fn world(&self) -> &World {
        &self.world
    }

    fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }
}
