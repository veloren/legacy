// Standard
use std::sync::{Arc, RwLock};

// Library
use coord::prelude::*;

// Project
use common::{
    get_version,
    net::{ClientMessage, ServerMessage},
};
use region::Entity;

// Local
use Client;
use ClientStatus;
use Payloads;

impl<P: Payloads> Client<P> {
    pub(crate) fn update_server(&self) {
        // Update the server with information about the player
        if let Some(player_entity) = self.player_entity() {
            let player_entity = player_entity.read().unwrap();
            self.conn.send(ClientMessage::PlayerEntityUpdate {
                pos: *player_entity.pos(),
                vel: *player_entity.vel(),
                ctrl_acc: *player_entity.ctrl_acc(),
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
                            self.add_entity(
                                uid,
                                Entity::new(
                                    vec3!(0.0, 0.0, 0.0),
                                    vec3!(0.0, 0.0, 0.0),
                                    vec3!(0.0, 0.0, 0.0),
                                    vec2!(0.0, 0.0),
                                ),
                            );
                            self.player_mut().control_entity(uid);
                        }
                    }
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
            },
            ServerMessage::Shutdown => self.set_status(ClientStatus::Disconnected),
            ServerMessage::RecvChatMsg { alias, msg } => self.callbacks().call_recv_chat_msg(&alias, &msg),
            ServerMessage::EntityUpdate {
                uid,
                pos,
                vel,
                ctrl_acc,
                look_dir,
            } => match self.entity(uid) {
                Some(entity) => {
                    let mut entity = entity.write().unwrap();
                    *entity.pos_mut() = pos;
                    *entity.vel_mut() = vel;
                    *entity.ctrl_acc_mut() = ctrl_acc;
                    *entity.look_dir_mut() = look_dir;
                },
                None => {
                    self.add_entity(uid, Entity::new(pos, vel, ctrl_acc, look_dir));
                },
            },
            ServerMessage::Ping => {
                self.conn.send(ClientMessage::Pong);
            },
            _ => {},
        }
    }
}
