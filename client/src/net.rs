// Standard
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
    thread,
};

// Library
use vek::*;
use parking_lot::Mutex;

// Project
use common::{
    get_version,
    msg::{SessionKind, ClientMsg, ServerMsg, ClientPostOffice, ClientPostBox},
    post::Incoming,
    manager::Manager,
};
use region::Entity;

// Local
use Client;
use ClientStatus;
use Payloads;

// Constants
const PING_TIMEOUT: Duration = Duration::from_secs(10);
const PING_FREQ: Duration = Duration::from_secs(2);

impl<P: Payloads> Client<P> {
    pub(crate) fn handle_incoming(&self, mgr: &mut Manager<Self>) {
        while let Ok(incoming) = self.postoffice.await_incoming() {
            match incoming {
                // Sessions
                Incoming::Session(session) => match session.kind {
                    SessionKind::Ping => {
                        let pb = Mutex::new(session.postbox);
                        Manager::add_worker(mgr, |client, running, _| {
                            client.handle_ping_session(pb.into_inner(), running);
                        })
                    },
                    _ => {},
                },

                // One-shot messages
                Incoming::Msg(ServerMsg::ChatMsg { text }) => {
                    self.callbacks().call_recv_chat_msg(&text)
                },
                Incoming::Msg(ServerMsg::EntityUpdate { uid, pos, vel, ori }) => match self.entity(uid) {
                    Some(entity) => {
                        let mut entity = entity.write();
                        *entity.pos_mut() = pos;
                        *entity.vel_mut() = vel;
                        *entity.ctrl_acc_mut() = Vec3::zero();
                        *entity.look_dir_mut() = Vec2::unit_y();
                    },
                    None => {
                        self.add_entity(uid, Entity::new(pos, vel, Vec3::zero(), Vec2::unit_y()));
                    },
                },
                Incoming::Msg(_) => {},

                // End
                Incoming::End => {}, // TODO: Something here
            }
        }

        *self.status.write() = ClientStatus::Disconnected;
    }

    fn handle_ping_session(&self, pb: ClientPostBox, running: &AtomicBool) {
        while running.load(Ordering::Relaxed) {
            thread::sleep(PING_FREQ);
            pb.send(ClientMsg::Ping);

            match pb.recv_timeout(PING_TIMEOUT) {
                Ok(ServerMsg::Ping) => {},
                _ => break, // Anything other than a ping     over this session is invalid
            }
        }
    }

    pub(crate) fn update_server(&self) {
        // Update the server with information about the player
        if let Some(player_entity) = self.player_entity() {
            let player_entity = player_entity.read();
            self.postoffice.send_one(ClientMsg::PlayerEntityUpdate {
                pos: *player_entity.pos(),
                vel: *player_entity.vel(),
                ori: Quaternion::identity(),
            });
        }
    }

    /*
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
                    let mut entity = entity.write();
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
    */
}
