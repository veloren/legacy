// Library
use coord::prelude::*;

// Project
use common::get_version;
use common::net::{Connection, ServerMessage, ClientMessage, Callback, UdpMgr};
use region::{Entity};

// Local
use {Client, Payloads, ClientStatus};

impl<P: Payloads> Client<P> {
    pub(crate) fn update_server(&self) {
        // Update the server with information about the player
        if let Some(uid) = self.player().entity_uid {
            if let Some(player_entity) = self.entities().get(&uid) {
                self.conn.send(ClientMessage::PlayerEntityUpdate {
                    pos: *player_entity.pos(),
                    vel: *player_entity.vel(),
                    ctrl_vel: *player_entity.ctrl_vel(),
                    look_dir: *player_entity.look_dir(),
                });
            }
        }
    }

    pub(crate) fn handle_packet(&self, packet: ServerMessage) {
        match packet {
            ServerMessage::Connected { entity_uid, version } => {
                if version == get_version() {
                    if let Some(uid) = entity_uid {
                        if !self.entities().contains_key(&uid) {
                            self.entities_mut().insert(uid, Entity::new(vec3!(0.0, 0.0, 0.0), vec3!(0.0, 0.0, 0.0), vec3!(0.0, 0.0, 0.0), vec2!(0.0, 0.0)));
                        }
                    }
                    self.player_mut().entity_uid = entity_uid;
                    self.set_status(ClientStatus::Connected);
                    info!("Connected!");
                } else {
                    warn!("Server version mismatch: server is version {}. Disconnected.", version);
                    self.set_status(ClientStatus::Disconnected);
                }
            },
            ServerMessage::Kicked { reason } => {
                warn!("Server kicked client for {}", reason);
                self.set_status(ClientStatus::Disconnected);
            }
            ServerMessage::Shutdown => self.set_status(ClientStatus::Disconnected),
            ServerMessage::RecvChatMsg { alias, msg } => self.callbacks().call_recv_chat_msg(&alias, &msg),
            ServerMessage::EntityUpdate { uid, pos, vel, ctrl_vel, look_dir } => {
                let mut entities = self.entities_mut();
                match entities.get_mut(&uid) {
                    Some(e) => {
                        *e.pos_mut() = pos;
                        *e.vel_mut() = vel;
                        *e.ctrl_vel_mut() = ctrl_vel;
                        *e.look_dir_mut() = look_dir;
                    }
                    None => { entities.insert(uid, Entity::new(pos, vel, ctrl_vel, look_dir)); },
                }
            },
            ServerMessage::Ping => self.conn.send(ClientMessage::Ping),
            _ => {},
        }
    }
}
