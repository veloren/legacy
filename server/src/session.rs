// Standard
use std::{cell::Cell, net::TcpStream, sync::Arc, thread::JoinHandle};

// Library
use bifrost::Relay;

// Project
use common::{
    net::{ClientMessage, Connection, ServerMessage, UdpMgr},
    Uid,
};

// Local
use network::event::{KickSession, PacketReceived};
use server_context::ServerContext;

#[derive(Copy, Clone, PartialEq)]
pub enum SessionState {
    Connected,
    ShouldKick,
}

pub const SESSION_TIMEOUT_SEC: i64 = 10;

pub struct Session {
    id: u32,
    listen_thread_handle: Option<JoinHandle<()>>,
    conn: Arc<Connection<ClientMessage>>,
    player_id: Option<Uid>,
    state: Cell<SessionState>,
    expire_at: i64,
}

impl Session {
    pub fn new(id: u32, stream: TcpStream, udpmgr: Arc<UdpMgr>, relay: &Relay<ServerContext>) -> Session {
        let relay = relay.clone();
        let conn = Connection::new_stream(
            stream,
            Box::new(move |m| {
                //callback message
                match m {
                    Ok(message) => {
                        relay.send(PacketReceived {
                            session_id: id,
                            data: message,
                        });
                    },
                    _ => {
                        relay.send(KickSession { session_id: id });
                    },
                };
            }),
            None,
            udpmgr,
        ).unwrap();
        Connection::start(&conn);
        let session = Session {
            id,
            listen_thread_handle: None,
            conn,
            player_id: None,
            state: Cell::new(SessionState::Connected),
            expire_at: time::now().to_timespec().sec + SESSION_TIMEOUT_SEC,
        };

        return session;
    }

    pub fn send_message(&self, message: ServerMessage) {
        self.conn.send(message);
        /*
        match self.packet_sender.borrow_mut().send_packet(packet) {
            Ok(_) => {},
            Err(_) => self.state.set(SessionState::ShouldKick),
        }*/
    }

    pub fn stop_conn(&self) { Connection::stop(&self.conn); }

    pub fn kick(&self) { self.state.set(SessionState::ShouldKick); }
    pub fn should_kick(&self) -> bool { self.state.get() == SessionState::ShouldKick }

    #[allow(dead_code)]
    pub fn get_id(&self) -> u32 { self.id }

    #[allow(dead_code)]
    pub fn set_player_id(&mut self, player_id: Option<Uid>) { self.player_id = player_id }
    #[allow(dead_code)]
    pub fn get_player_id(&self) -> Option<Uid> { self.player_id }
    #[allow(dead_code)]
    pub fn has_player(&self) -> bool { self.player_id.is_some() }

    #[allow(dead_code)]
    pub fn is_alive(&self) -> bool { time::now().to_timespec().sec < self.expire_at }
    #[allow(dead_code)]
    pub fn keep_alive(&mut self) { self.expire_at = time::now().to_timespec().sec + SESSION_TIMEOUT_SEC; }
}
