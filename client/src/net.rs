// Standard
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

// Library
use parking_lot::Mutex;
use vek::*;

// Project
use common::{
    get_version,
    manager::Manager,
    msg::{ClientMsg, ClientPostBox, ClientPostOffice, ServerMsg, SessionKind},
    post::Incoming,
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
                            thread::spawn(move || {
                                let pb = pb.into_inner();

                                loop {
                                    thread::sleep(PING_FREQ);
                                    pb.send(ClientMsg::Ping);

                                    match pb.recv_timeout(PING_TIMEOUT) {
                                        Ok(ServerMsg::Ping) => {},
                                        _ => break, // Anything other than a ping over this session is invalid
                                    }
                                }
                            });
                        })
                    },
                    _ => {},
                },

                // One-shot messages
                Incoming::Msg(ServerMsg::ChatMsg { text }) => self.callbacks().call_recv_chat_msg(&text),
                Incoming::Msg(ServerMsg::EntityUpdate {
                    uid,
                    pos,
                    vel,
                    ctrl_dir,
                }) => {
                    match self.entity(uid) {
                        Some(entity) => {
                            // Ignore the update if it's for the player's own entity, unless it's a big jump
                            if self.player.read().entity_uid().map(|u| u != uid).unwrap_or(true) || self
                                .player_entity()
                                .map(|e| e.read().pos().distance(pos) > 5.0)
                                .unwrap_or(true)
                            {
                                let mut entity = entity.write();
                                *entity.pos_mut() = pos;
                                *entity.vel_mut() = vel;
                                *entity.look_dir_mut() = ctrl_dir;
                            }
                        },
                        None => {
                            self.add_entity(uid, Entity::new(pos, vel, Vec3::zero(), Vec2::unit_y()));
                        },
                    }
                },
                Incoming::Msg(_) => {},

                // End
                Incoming::End => {}, // TODO: Something here
            }
        }

        *self.status.write() = ClientStatus::Disconnected;
    }

    pub(crate) fn update_server(&self) {
        // Update the server with information about the player
        if let Some(player_entity) = self.player_entity() {
            let player_entity = player_entity.read();
            self.postoffice.send_one(ClientMsg::PlayerEntityUpdate {
                pos: *player_entity.pos(),
                vel: *player_entity.vel(),
                ctrl_dir: *player_entity.look_dir(),
            });
        }
    }
}
