// Library
use specs::prelude::*;

// Project
use common::msg::ServerMsg;

// Local
use Payloads;
use Server;
use net::{Client, DisconnectReason};
use player::Player;

pub trait Api {
    fn disconnect_player(&mut self, player: Entity, reason: DisconnectReason);
    fn send_chat_msg(&self, player: Entity, text: &str);
    fn send_net_msg(&self, player: Entity, msg: ServerMsg);
    fn broadcast_chat_msg(&self, text: &str);
    fn broadcast_net_msg(&self, msg: ServerMsg);

    fn world(&self) -> &World;
    fn world_mut(&mut self) -> &mut World;
}

impl<P: Payloads> Api for Server<P> {
    fn disconnect_player(&mut self, player: Entity, reason: DisconnectReason) {
        // Stop the postoffice
        if let Some(client) = self.world.read_storage::<Client>().get(player) {
            let _ = client.postoffice.stop(); // We don't care if this fails
        }

        if let Some(player_comp) = self.world.read_storage::<Player>().get(player) {
            self.broadcast_chat_msg(&format!("[{} disconnected: {}]", player_comp.alias, reason));
            self.payload.on_player_disconnect(self, player, reason);
        }

        let _ = self.world.delete_entity(player);
    }

    fn send_chat_msg(&self, player: Entity, text: &str) {
        self.send_net_msg(player, ServerMsg::ChatMsg { text: text.to_string() });
    }

    fn send_net_msg(&self, player: Entity, msg: ServerMsg) {
        if let Some(client) = self.world.read_storage::<Client>().get(player) {
            let _ = client.postoffice.send_one(msg.clone()); // We don't care if this fails
        }
    }

    fn broadcast_chat_msg(&self, text: &str) {
        self.broadcast_net_msg(ServerMsg::ChatMsg { text: text.to_string() });
    }

    fn broadcast_net_msg(&self, msg: ServerMsg) {
        let clients = self.world.read_storage::<Client>();
        for entity in self.world.entities().join() {
            if let Some(client) = clients.get(entity) {
                let _ = client.postoffice.send_one(msg.clone()); // We don't care if this fails
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
