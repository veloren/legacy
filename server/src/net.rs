// Standard
use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
    thread,
    fmt,
};

// Library
use specs::{
    Component, VecStorage, Entity, Builder, Join,
    saveload::Marker,
};

// Project
use common::{
    manager::Manager,
    msg::{SessionKind, ClientMsg, ServerMsg, ServerPostOffice, ServerPostBox, PlayMode},
    post::Incoming,
    Uid,
};
use region::{
    ecs, ecs::{
        CreateUtil,
        phys::{Pos, Vel, Ori},
        net::SyncMarker,
    },
};

// Local
use Payloads;
use Server;
use Wrapper;
use api::Api;
use Error;
use player::Player;

// Constants
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const PING_TIMEOUT: Duration = Duration::from_secs(10);
const PING_FREQ: Duration = Duration::from_secs(2);

// Server

#[derive(Debug)]
pub struct Client {
    pub postoffice: Arc<Manager<ServerPostOffice>>,
}

impl Component for Client {
    type Storage = VecStorage<Self>;
}

// DisconnectReason

pub enum DisconnectReason {
    Logout,
    Timeout,
    Kicked(String),
}

impl fmt::Display for DisconnectReason {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match self {
            DisconnectReason::Logout => format!("Logout"),
            DisconnectReason::Timeout => format!("Timedout"),
            DisconnectReason::Kicked(msg) => format!("Kicked ({})", msg),
        })
    }
}

// Authenticate a client. If authentication is successful,
pub(crate) fn auth_client<P: Payloads>(srv: &Wrapper<Server<P>>, po: Manager<ServerPostOffice>) -> Result<Entity, Error> {
    // Perform a connection handshake. If everything works out, create the player
    // First, wait for the correct `Connect` session
    if let Ok(Incoming::Session(session)) = po.await_incoming() {
        if let SessionKind::Connect = session.kind {
            // Wait for the appropriate `Connect` message
            if let Ok(ClientMsg::Connect { alias, mode }) = session.postbox.recv_timeout(CONNECT_TIMEOUT) {
                // Create the player's entity and return it
                let (player, player_uid) = srv.do_for_mut(|srv| {
                    // Notify all other players
                    srv.broadcast_chat_msg(&format!("[{} has joined the server]", alias));

                    // Create a new player
                    let player = srv.create_player(alias.clone(), mode, po).build();

                    // Run the connecting player past the payload interface
                    srv.payload.on_player_connect(srv, player);

                    // Find the uid for the player's character entity (if the player has a character)
                    let player_uid = srv.world.read_storage::<SyncMarker>().get(player).map(|sm| sm.id());
                    (player, player_uid)
                });

                // Inform the client that they've successfully connected
                session.postbox.send(ServerMsg::Connected {
                    player_uid,
                });

                Ok(player)
            } else {
                Err(Error::NoConnectMsg)
            }
        } else {
            Err(Error::InvalidConnectSession)
        }
    } else {
        Err(Error::NoConnectSession)
    }
}

pub(crate) fn handle_player_post<P: Payloads>(srv: &Wrapper<Server<P>>, player: Entity, running: &AtomicBool, mut mgr: Manager<Wrapper<Server<P>>>) {
    // Ping worker
    Manager::add_worker(&mut mgr, move |srv, running, _| {
        if let Some(pb) = srv.do_for(|srv| srv
            .world
            .read_storage::<Client>()
            .get(player)
            .map(|p| p.postoffice.create_postbox(SessionKind::Ping))
        ) {
            // Wait for pings, respond with another ping
            while running.load(Ordering::Relaxed) {
                thread::sleep(PING_FREQ);
                pb.send(ServerMsg::Ping);
                match pb.recv_timeout(PING_TIMEOUT) {
                    Ok(ClientMsg::Ping) => {},
                    _ => break, // Anything other than a ping over this session is invalid
                }
            }

            // Kick the player if the ping expires
            srv.do_for_mut(|srv| srv.disconnect_player(player, DisconnectReason::Timeout));
        }
    });

    // Await incoming sessions and one-shot messages
    if let Some(po) = srv.do_for(|srv| srv.world.read_storage::<Client>().get(player).map(|p| p.postoffice.clone())) {
        while let Ok(msg) = po.await_incoming() {
            match msg {
                Incoming::Session(session) => {}, // TODO: Something here
                Incoming::Msg(msg) => handle_oneshot(srv, msg, player, &mgr),
                Incoming::End => break,
            }
        }
    }

    // Disconnect the client
    srv.do_for_mut(|srv| srv.disconnect_player(player, DisconnectReason::Logout));
}

pub(crate) fn handle_oneshot<P: Payloads>(srv: &Wrapper<Server<P>>, msg: ClientMsg, player: Entity, mgr: &Manager<Wrapper<Server<P>>>) {
    match msg {
        ClientMsg::ChatMsg { text } => process_chat_msg(srv, text, player, mgr),
        ClientMsg::PlayerEntityUpdate { pos, vel, ori } => {
            // Update the player's entity
            srv.do_for_mut(|srv| srv.update_player_entity(player, pos, vel, ori));
        }
        _ => {},
    }
}

pub(crate) fn process_chat_msg<P: Payloads>(srv: &Wrapper<Server<P>>, text: String, player: Entity, mgr: &Manager<Wrapper<Server<P>>>) {
    if text.starts_with('/') {
        match text.split(' ').next().map(|s| s.get(1..)) {
            Some(Some("help")) => srv.do_for(|srv| {
                srv.send_chat_msg(player, "Available commands:");
                srv.send_chat_msg(player, "/players - View all online players");
                srv.send_chat_msg(player, "/tp <alias> - Teleport to a player");
            }),
            Some(Some("players")) => srv.do_for(|srv| {
                // Find a list of player names
                let player_names = srv.world.read_storage::<Player>().join().map(|p| p.alias.clone()).collect::<Vec<_>>().join(", ");
                srv.send_chat_msg(player, &format!("Online Players: {}", player_names));
            }),
            Some(Some("tp")) => { // TODO: Simplify this? Put it somewhere else?
                // Find the name the player typed (i.e: '/tp zesterer')
                if let Some(tgt_player) = text.split(' ').nth(1) {
                    let tgt_pos = srv.do_for(|srv| {
                        // Find the position of that player
                        let pos_storage = srv.world.read_storage::<Pos>();
                        let player_storage = srv.world.read_storage::<Player>();
                        (&pos_storage, &player_storage).join().find_map(|(pos, player)| {
                            if player.alias == tgt_player {
                                Some(pos.0)
                            } else {
                                None
                            }
                        })
                    });

                    // If a position was found, teleport to it
                    if let Some(pos) = tgt_pos {
                        if let Some(()) = srv.do_for_mut(|srv| srv.world.write_storage::<Pos>().get_mut(player).map(|p| p.0 = pos)) {
                            srv.do_for(|srv| srv.send_chat_msg(player, &format!("Teleported to {}!", tgt_player)));
                        } else {
                            srv.do_for(|srv| srv.send_chat_msg(player, "You don't have a position!"));
                        }
                    } else {
                        srv.do_for(|srv| srv.send_chat_msg(player, &format!("Could not locate {}!", tgt_player)));
                    }
                } else {
                    srv.do_for(|srv| srv.send_chat_msg(player, "Usage: /tp <alias>"));
                }
            },
            _ => srv.do_for(|srv| srv.send_chat_msg(player, "Unrecognised command!")),
        }
    } else if let Some(text) = srv.do_for(|srv| srv.payload.on_chat_msg(srv, player, &text)) { // Run the message past the payload interface
        srv.do_for(|srv| srv.broadcast_chat_msg(&text));
    }
}

impl<P: Payloads> Server<P> {
    pub(crate) fn sync_players(&self) {
        let pos_storage = self.world.read_storage::<Pos>();
        let vel_storage = self.world.read_storage::<Vel>();
        let ori_storage = self.world.read_storage::<Ori>();
        let sync_storage = self.world.read_storage::<SyncMarker>();
        for (sync_storage, pos, vel, ori) in (&sync_storage, &pos_storage, &vel_storage, &ori_storage).join() {
            self.broadcast_net_msg(ServerMsg::EntityUpdate {
                uid: sync_storage.id(),
                pos: pos.0,
                vel: vel.0,
                ori: ori.0,
            });
        }
    }
}
