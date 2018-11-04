// Standard
use std::{
    fmt,
    sync::{atomic::Ordering, Arc},
    thread,
    time::Duration,
};

// Library
use specs::{saveload::Marker, Builder, Component, Entity, Join, VecStorage};

// Project
use common::{
    ecs::{
        net::UidMarker,
        phys::{Dir, Pos, Vel},
        NetComp,
    },
    util::{
        manager::Manager,
        msg::{ClientMsg, ServerMsg, ServerPostOffice, SessionKind},
        post::Incoming,
    },
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
    let _ = session.postbox.send(ServerMsg::Connected {
        player_uid,
        time: srv.do_for(|srv| srv.clock_tick_time),
    });

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
    /// Update the value of a component. Returns `true` if the component exists, and `false` otherwise.
    #[allow(dead_code)]
    pub(crate) fn update_comp<T: NetComp + Clone>(&mut self, entity: Entity, comp: T) -> bool {
        self.world
            .write_storage::<T>()
            .get_mut(entity)
            .map(|c| *c = comp)
            .is_some()
    }

    /// Apply an operation to a component mutably. If the component does not exist, this operation will not occur.
    #[allow(dead_code)]
    pub(crate) fn do_for_comp_mut<T: NetComp + Clone, R, F: FnOnce(&mut T) -> R>(
        &mut self,
        entity: Entity,
        f: F,
    ) -> Option<R> {
        self.world.write_storage::<T>().get_mut(entity).map(|c| f(c))
    }

    /// Retrieve component immutably. If the component does not exist, this operation will not occur.
    pub(crate) fn do_for_comp<T: NetComp + Clone, R, F: FnOnce(&T) -> R>(&self, entity: Entity, f: F) -> Option<R> {
        self.world.read_storage::<T>().get(entity).map(|c| f(c))
    }

    /// Update clients of a component's value, excepting those clients for whom that component is attributed
    /// (e.g: a client won't get it's own player position sent back to it)
    #[allow(dead_code)]
    pub(crate) fn notify_comp<T: NetComp>(&self, entity: Entity) {
        // Convert the component (if it exists and if it support it) to a CompStore
        let store = if let Some(Some(s)) = self.world.read_storage::<T>().get(entity).map(|c| c.to_store()) {
            s
        } else {
            return;
        };

        // Find the UID of the entity we're notifying clients of
        let entity_uid = if let Some(u) = self.world.read_storage::<UidMarker>().get(entity) {
            u.id()
        } else {
            return;
        };

        // Send the store to all clients that need it
        for (client_uid, client) in (
            &self.world.read_storage::<UidMarker>(),
            &self.world.read_storage::<Client>(),
        )
            .join()
        {
            let client_uid = client_uid.id();

            // Don't notify a client of information concerning itself
            if client_uid != entity_uid {
                let _ = client.postoffice.send_one(ServerMsg::CompUpdate {
                    uid: entity_uid,
                    store: store.clone(),
                });
            }
        }
    }

    /// Update *all* clients of a component's value, overriding any other values a client may have had
    #[allow(dead_code)]
    pub(crate) fn force_comp<T: NetComp + Clone>(&self, entity: Entity) {
        // Convert the component (if it exists and if it support it) to a CompStore
        let store = if let Some(Some(s)) = self.world.read_storage::<T>().get(entity).map(|c| c.to_store()) {
            s
        } else {
            return;
        };

        // Find the UID of the entity we're notifying clients of
        let entity_uid = if let Some(u) = self.world.read_storage::<UidMarker>().get(entity) {
            u.id()
        } else {
            return;
        };

        // Send the store to all clients
        self.broadcast_net_msg(ServerMsg::CompUpdate {
            uid: entity_uid,
            store: store.clone(),
        });
    }

    pub(crate) fn sync_players(&self) {
        // For each entity in the world...
        // TODO: Add a notion of range? Don't update clients of entities that are nowhere near them
        for entity in self.world.entities().join() {
            // Notify clients of the following components...
            self.notify_comp::<Pos>(entity);
            self.notify_comp::<Vel>(entity);
            self.notify_comp::<Dir>(entity);
        }
    }

    pub(crate) fn sync_player_time(&self) { self.broadcast_net_msg(ServerMsg::TimeUpdate(self.clock_tick_time)); }
}
