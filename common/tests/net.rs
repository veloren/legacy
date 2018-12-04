// Standard
use std::{net::TcpListener, sync::Arc, thread, time::Duration};

// Library
use serde_derive::{Deserialize, Serialize};

// Project
use common::{
    net::Message,
    util::{
        manager::Manager,
        post::{Incoming, PostBox, PostOffice},
        testutils::PORTS,
    },
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
    let server_addr = PORTS.next();
    let listener = TcpListener::bind(&server_addr).unwrap();
    thread::spawn(move || match listener.incoming().next() {
        Some(Ok(stream)) => {
            thread::spawn(move || handle_client(PostOffice::to_client(stream).unwrap()));
        },
        Some(Err(e)) => panic!("Connection error: {}", e),
        None => panic!("No client received"),
    });

    // Client
    handle_remote(PostOffice::to_server(&server_addr).unwrap());
}

fn handle_client(postoffice: Manager<PostOffice<SessionKind, ServerMsg, ClientMsg>>) {
    while let Ok(Incoming::Session(s)) = postoffice.await_incoming() {
        match s.kind {
            SessionKind::PingPong => thread::spawn(move || handle_pingpong(s.postbox)),
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
