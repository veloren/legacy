// Standard
use std::sync::{Arc, RwLock};

// Library
use coord::prelude::*;

// Project
use common::get_version;
use common::net::{ServerMessage, ClientMessage};
use region::{Entity};

// Local
use {Client, Payloads, ClientStatus};

impl<P: Payloads> Client<P> {
    pub(crate) fn update_server(&self) {
        if let Some(player_entity) = self.player_entity() {
            // Nothing yet
        }
        // Update the server with information about the player
        if let Some(player_entity) = self.player_entity() {
            let player_entity = player_entity.write().unwrap();
            self.conn.send(ClientMessage::PlayerEntityUpdate {
                pos: *player_entity.pos(),
                vel: *player_entity.vel(),
                ctrl_vel: *player_entity.ctrl_vel(),
                look_dir: *player_entity.look_dir(),
            });
        }
    }

    pub(crate) fn handle_packet(&self, packet: ServerMessage) {
        match packet {
            ServerMessage::Connected { entity_uid, version } => {
                if version == get_version() {
                    if let Some(uid) = entity_uid {
                        if !self.entities().contains_key(&uid) {
                            self.entities_mut().insert(uid, Arc::new(RwLock::new(Entity::new(vec3!(0.0, 0.0, 0.0), vec3!(0.0, 0.0, 0.0), vec3!(0.0, 0.0, 0.0), vec2!(0.0, 0.0)))));
                        }
                    }
                    entity_uid.map(|uid| self.player_mut().control_entity(uid));
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
                    Some(entity) => {
                        let mut entity = entity.write().unwrap();
                        *entity.pos_mut() = pos;
                        *entity.vel_mut() = vel;
                        *entity.ctrl_vel_mut() = ctrl_vel;
                        *entity.look_dir_mut() = look_dir;
                    }
                    None => { entities.insert(uid, Arc::new(RwLock::new(Entity::new(pos, vel, ctrl_vel, look_dir)))); },
                }
            },
            ServerMessage::Ping => self.conn.send(ClientMessage::Ping),
            _ => {},
        }
    }
}
