// Standard
use std::{
    fmt,
    sync::{atomic::Ordering, Arc},
    thread,
    time::Duration,
};

// Library
use specs::{saveload::Marker, Builder, Component, Entity, Join, VecStorage};
use vek::*;

// Project
use common::{
    manager::Manager,
    msg::{ClientMsg, ServerMsg, ServerPostOffice, SessionKind},
    post::Incoming,
};
use region::ecs::{
    net::UidMarker,
    phys::{Dir, Pos, Vel},
    NetComp,
};

// Local
use api::Api;
use msg::process_chat_msg;
use Error;
use Payloads;
use Server;
use Wrapper;

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
        write!(
            f,
            "{}",
            match self {
                DisconnectReason::Logout => format!("Logout"),
                DisconnectReason::Timeout => format!("Timedout"),
                DisconnectReason::Kicked(msg) => format!("Kicked ({})", msg),
            }
        )
    }
}

// Authenticate a client. If authentication is successful,
pub(crate) fn auth_client<P: Payloads>(
    srv: &Wrapper<Server<P>>,
    po: Manager<ServerPostOffice>,
) -> Result<Entity, Error> {
    // Perform a connection handshake. If everything works out, create the player
    // First, wait for the correct `Connect` session
    let session = if let Ok(Incoming::Session(s)) = po.await_incoming() {
        s
    } else {
        return Err(Error::NoConnectSession);
    };

    // Verify that the first session is a SessionKind::Connect
    if let SessionKind::Connect = session.kind {
    } else {
        return Err(Error::InvalidConnectSession);
    }

    // Wait for a ClientMsg::Connect, thereby committing the client to connecting
    let (alias, mode) = if let Ok(ClientMsg::Connect { alias, mode }) = session.postbox.recv_timeout(CONNECT_TIMEOUT) {
        (alias, mode)
    } else {
        return Err(Error::NoConnectMsg);
    };

    // Create the player's entity and return it
    let (player, player_uid) = srv.do_for_mut(|srv| {
        // Notify all other players
        srv.broadcast_chat_msg(&format!("[{} has joined the server]", alias));

        // Create a new player
        let player = srv.create_player(alias.clone(), mode, po).build();

        // Force an update to the player position to inform them where they are
        srv.force_comp::<Pos>(player);

        // Run the connecting player past the payload interface
        srv.payload.on_player_connect(srv, player);

        // Find the uid for the player's character entity (if the player has a character)
        let player_uid = srv.world.read_storage::<UidMarker>().get(player).map(|sm| sm.id());
        (player, player_uid)
    });

    // Inform the client that they've successfully connected
    let _ = session.postbox.send(ServerMsg::Connected { player_uid });

    Ok(player)
}

pub(crate) fn handle_player_post<P: Payloads>(
    srv: &Wrapper<Server<P>>,
    player: Entity,
    mut mgr: Manager<Wrapper<Server<P>>>,
) {
    // Ping worker
    Manager::add_worker(&mut mgr, move |srv, running, _| {
        if let Some(pb) = srv.do_for(|srv| {
            srv.world
                .read_storage::<Client>()
                .get(player)
                .map(|p| p.postoffice.create_postbox(SessionKind::Ping))
        }) {
            // Wait for pings, respond with another ping
            while running.load(Ordering::Relaxed) {
                thread::sleep(PING_FREQ);

                // Send a ping response
                if let Err(_) = pb.send(ServerMsg::Ping) {
                    break;
                }

                // Await a ping response from the client
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
    if let Some(po) = srv.do_for(|srv| {
        srv.world
            .read_storage::<Client>()
            .get(player)
            .map(|p| p.postoffice.clone())
    }) {
        while let Ok(msg) = po.await_incoming() {
            match msg {
                Incoming::Session(_session) => {}, // TODO: Something here
                Incoming::Msg(msg) => handle_oneshot(srv, msg, player, &mgr),
                Incoming::End => break,
            }
        }
    }

    // Disconnect the client
    srv.do_for_mut(|srv| srv.disconnect_player(player, DisconnectReason::Logout));
}

pub(crate) fn handle_oneshot<P: Payloads>(
    srv: &Wrapper<Server<P>>,
    msg: ClientMsg,
    player: Entity,
    mgr: &Manager<Wrapper<Server<P>>>,
) {
    match msg {
        ClientMsg::ChatMsg { text } => process_chat_msg(srv, text, player, mgr),
        ClientMsg::PlayerEntityUpdate { pos, vel, dir } => {
            // Update the player's entity
            srv.do_for_mut(|srv| {
                srv.update_comp(player, Pos(pos));
                srv.update_comp(player, Vel(vel));
                srv.update_comp(player, Dir(dir));
            });
        },
        _ => {},
    }
}

impl<P: Payloads> Server<P> {
    pub(crate) fn update_comp<T: NetComp + Clone>(&mut self, entity: Entity, comp: T) -> bool {
        self.world
            .write_storage::<T>()
            .get_mut(entity)
            .map(|c| *c = comp)
            .is_some()
    }

    pub(crate) fn do_for_comp_mut<T: NetComp + Clone, R, F: FnOnce(&mut T) -> R>(
        &mut self,
        entity: Entity,
        f: F,
    ) -> Option<R> {
        self.world.write_storage::<T>().get_mut(entity).map(|c| f(c))
    }

    // Update clients of a component's value, excepting those clients for whom that component is attributed
    // (e.g: a client won't get it's own player position sent back to it)
    pub(crate) fn notify_comp<T: NetComp>(&mut self, entity: Entity) {
        if let Some(Some(store)) = self.world.read_storage::<T>().get(entity).map(|c| c.to_store()) {
            if let Some(entity_uid) = self.world.read_storage::<UidMarker>().get(entity) {
                for (entity, client_uid, client) in (
                    &self.world.entities(),
                    &self.world.read_storage::<UidMarker>(),
                    &self.world.read_storage::<Client>(),
                )
                    .join()
                {
                    let entity_uid = entity_uid.id();
                    let client_uid = client_uid.id();

                    // Don't notify a client of information concerning itself
                    if client_uid != entity_uid {
                        client.postoffice.send_one(ServerMsg::CompUpdate {
                            uid: entity_uid,
                            store: store.clone(),
                        });
                    }
                }
            }
        }
    }

    // Update *all* clients of a component's value, overriding any other values a client may have had
    pub(crate) fn force_comp<T: NetComp + Clone>(&mut self, entity: Entity) {
        if let Some(Some(store)) = self.world.read_storage::<T>().get(entity).map(|c| c.to_store()) {
            if let Some(entity_uid) = self.world.read_storage::<UidMarker>().get(entity) {
                let entity_uid = entity_uid.id();

                self.broadcast_net_msg(ServerMsg::CompUpdate {
                    uid: entity_uid,
                    store: store.clone(),
                });
            }
        }
    }

    pub(crate) fn sync_players(&self) {
        for (uid, pos, vel, dir) in (
            &self.world.read_storage::<UidMarker>(),
            &self.world.read_storage::<Pos>(),
            &self.world.read_storage::<Vel>(),
            &self.world.read_storage::<Dir>(),
        )
            .join()
        {
            // Find the UID of the entity that is having its details sent to clients
            let entity_uid = uid.id();

            for (entity, uid, client) in (
                &self.world.entities(),
                &self.world.read_storage::<UidMarker>(),
                &self.world.read_storage::<Client>(),
            )
                .join()
            {
                let client_uid = uid.id();
                // Don't send a client information they already know about themselves
                if client_uid != entity_uid {
                    // These unwraps are verifably SAFE and will be elided at compile-time.
                    // *Every one* of these types implements NetComp and produces a Some(CompStore)
                    // We simply use .unwrap() here to avoid a heap of if-lets
                    // TODO: Add tests to verify this
                    let _ = client.postoffice.send_one(ServerMsg::CompUpdate {
                        uid: entity_uid,
                        store: pos.to_store().unwrap(),
                    });
                    let _ = client.postoffice.send_one(ServerMsg::CompUpdate {
                        uid: entity_uid,
                        store: vel.to_store().unwrap(),
                    });
                    let _ = client.postoffice.send_one(ServerMsg::CompUpdate {
                        uid: entity_uid,
                        store: dir.to_store().unwrap(),
                    });
                }
            }
        }
    }
}
