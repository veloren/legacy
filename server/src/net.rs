// Standard
use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
    thread,
};

// Project
use common::{
    manager::Manager,
    msg::{SessionKind, ClientMsg, ServerMsg, ServerPostOffice, ServerPostBox},
    post::Incoming,
    Uid,
};

// Local
use Payloads;
use Server;
use player::Player;

// Constants
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const PING_TIMEOUT: Duration = Duration::from_secs(10);
const PING_FREQ: Duration = Duration::from_secs(2);

impl<P: Payloads> Server<P> {
    pub(crate) fn handle_incoming(&self, po: Manager<ServerPostOffice>, running: &AtomicBool, mgr: Manager<Self>) {
        // Perform a connection handshake. If everything works out, create the player
        // First, wait for the correct `Connect` session
        if let Ok(Incoming::Session(session)) = po.await_incoming() {
            if let SessionKind::Connect = session.kind {
                // Wait for the appropriate `Connect` message
                if let Ok(ClientMsg::Connect { alias, mode }) = session.postbox.recv_timeout(CONNECT_TIMEOUT) {
                    // Generate a new UID for the player
                    let uid = self.gen_uid();

                    // Create a temporary player before adding to server records
                    let player = Arc::new(Player::new(uid, po, alias, mode));

                    // Run the connecting player past the payload interface
                    if self.payload.on_player_connect(&player) {
                        self.broadcast_msg(&format!("[{} has joined the server]", player.alias()));

                        // Add the player to the server records
                        self.players.write().unwrap().insert(uid, player);

                        // Inform the client that they've successfully connected
                        session.postbox.send(ServerMsg::Connected);

                        // Start handling them as a player, not simply as an incoming client
                        self.handle_player(uid, running, mgr);
                    }
                }
            }
        }
    }

    fn handle_player(&self, player_uid: Uid, running: &AtomicBool, mut mgr: Manager<Self>) {
        if let Some(player) = self.players.read().unwrap().get(&player_uid).map(|p| p.clone()) {
            // Ping worker
            let ping_pb = Mutex::new(player.postoffice().create_postbox(SessionKind::Ping));
            Manager::add_worker(&mut mgr, move |server, running, mut mgr| {
                let pb = ping_pb.into_inner().unwrap();
                while running.load(Ordering::Relaxed) {
                    thread::sleep(PING_FREQ);
                    pb.send(ServerMsg::Ping);

                    match pb.recv_timeout(PING_TIMEOUT) {
                        Ok(ClientMsg::Ping) => {},
                        _ => break, // Anything other than a ping     over this session is invalid
                    }
                }

                // Kick the player if the ping expires
                server.kick_player(player_uid, "Timeout");
            });

            while let Ok(incoming) = player.postoffice().await_incoming() {
                match incoming {
                    // Session
                    Incoming::Session(_) => {},

                    // One-shot
                    Incoming::Msg(msg) => self.handle_oneshot(msg, &player, &mgr),

                    // End
                    Incoming::End => break,
                }
            }

            // Inform the frontend that the player has disconnected
            self.payload.on_player_disconnect(&player);
        }

        // Remove the player when communications have ceased
        if let Some(player) = self.remove_player(player_uid) {
            self.broadcast_msg(&format!("[{} has left the server]", player.alias()));
        }
    }

    pub(crate) fn handle_oneshot(&self, msg: ClientMsg, player: &Arc<Player<<P as Payloads>::Player>>, mgr: &Manager<Self>) {
        match msg {
            ClientMsg::ChatMsg { text } => {
                // Run the message past the payload interface
                if let Some(text) = self.payload.on_chat_msg(player, &text) {
                    self.broadcast_msg(&text)
                }
            },
            _ => {},
        }
    }

    pub fn broadcast_msg(&self, text: &str) {
        for (uid, player) in self.players.read().unwrap().iter() {
            player.postoffice().send_one(ServerMsg::ChatMsg { text: text.into() });
        }
    }
}
