// Crates
extern crate common;
#[macro_use]
extern crate serde_derive;

// Standard
use std::{net::TcpListener, sync::Arc, thread, time::Duration};

// Project
use common::{
    net::Message,
    post::{PostBox, PostOffice},
    manager::Manager,
};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
enum ClientMsg {
    Ping,
}
impl Message for ClientMsg {}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
enum ServerMsg {
    Pong,
}
impl Message for ServerMsg {}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SessionKind {
    PingPong,
    // TODO: Implement other session type
}
impl Message for SessionKind {}

#[test]
fn post_office() {
    // Server
    let listener = TcpListener::bind("0.0.0.0:8888").unwrap();
    thread::spawn(move || match listener.incoming().next() {
        Some(Ok(stream)) => {
            thread::spawn(move || handle_client(Manager::init(PostOffice::to_client(stream).unwrap())));
        },
        Some(Err(e)) => panic!("Connection error: {}", e),
        None => panic!("No client received"),
    });

    // Client
    handle_remote(Manager::init(PostOffice::to_server("127.0.0.1:8888").unwrap()));
}

fn handle_client(postoffice: Manager<PostOffice<SessionKind, ServerMsg, ClientMsg>>) {
    while let Ok(session) = postoffice.await_incoming() {
        match session.kind {
            SessionKind::PingPong => thread::spawn(move || handle_pingpong(session.postbox)),
        };
    }
}

fn handle_pingpong(pb: PostBox<SessionKind, ServerMsg, ClientMsg>) {
    while let Ok(msg) = pb.recv() {
        assert_eq!(msg, ClientMsg::Ping);
        let _ = pb.send(ServerMsg::Pong);
    }
}

fn handle_remote(po: Manager<PostOffice<SessionKind, ClientMsg, ServerMsg>>) {
    let po = Arc::new(po);

    let po_ref = po.clone();
    thread::spawn(move || {
        while let Ok(_pb) = po_ref.await_incoming() {
            // Handle server sessions
        }
    });

    thread::sleep(Duration::from_millis(1000)); // Waiting for connection

    for _ in 0..10 {
        let pb_r = po.create_postbox(SessionKind::PingPong);

        let _ = pb_r.send(ClientMsg::Ping);
        let msg = pb_r.recv().unwrap();
        assert_eq!(ServerMsg::Pong, msg);

        let _ = pb_r.send(ClientMsg::Ping);
        let msg = pb_r.recv().unwrap();
        assert_eq!(ServerMsg::Pong, msg);
    }
}
