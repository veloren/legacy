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
                            client.handle_ping_session(pb.into_inner(), running);
                        })
                    },
                    _ => {},
                },

                // One-shot messages
                Incoming::Msg(ServerMsg::ChatMsg { text }) => self.callbacks().call_recv_chat_msg(&text),
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
}
