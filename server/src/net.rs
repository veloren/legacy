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
use specs::{Component, VecStorage, Entity, Builder};

// Project
use common::{
    manager::Manager,
    msg::{SessionKind, ClientMsg, ServerMsg, ServerPostOffice, ServerPostBox, PlayMode},
    post::Incoming,
    Uid,
};
use region::{
    ecs, ecs::CreateUtil,
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
                // Inform the client that they've successfully connected
                session.postbox.send(ServerMsg::Connected);

                // Create the player's entity and return it
                Ok(srv.do_for_mut(|srv| {
                    // Create a new player
                    let player = srv.create_player(alias.clone(), mode, po).build();

                    srv.broadcast_msg(&format!("[{} has joined the server]", alias));

                    // Run the connecting player past the payload interface
                    srv.payload.on_client_connect(srv, player);

                    player
                }))
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

// Server

pub(crate) fn handle_oneshot<P: Payloads>(srv: &Wrapper<Server<P>>, msg: ClientMsg, client: Entity, mgr: &Manager<Wrapper<Server<P>>>) {
    match msg {
        ClientMsg::ChatMsg { text } => {
            // Run the message past the payload interface
            if let Some(text) = srv.do_for(|srv| srv.payload.on_chat_msg(srv, client, &text)) {
                srv.do_for(|srv| srv.broadcast_msg(&text));
            }
        },
        _ => {},
    }
}
