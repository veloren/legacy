// Standard
use std::{
    collections::HashMap,
    net::{TcpStream, ToSocketAddrs},
    sync::{
        atomic::{AtomicU64, Ordering},
        mpsc::{self, RecvError, SendError},
        Arc, Mutex,
    },
    thread,
};

// Local
use net::{Connection, Error, Message, UdpMgr};
use session::SessionKind;

// Information
// -----------
//
// The post system is a one-to-many relationship between a single postoffice and many postboxes.
// We use mpscs to communicate between each: the outgoing mpsc is owned by the postoffice, the
// incoming mpscs are owned by each postbox respectively. When a message comes in, it gets sent
// off to the corresponding postbox's receiving mpsc. When a message gets sent from a postbox, it
// gets sent through the postoffice's outgoing mpsc.
//
//      ,--> PostBox
//     v
// PostOffice <---> PostBox
//     ^
//     `--> PostBox

// Letter

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Letter<M> {
    OpenBox { uid: u64, kind: SessionKind },
    CloseBox(u64),
    Message { uid: u64, payload: M },
}

impl<M: Message> Message for Letter<M> {}

// PostBoxSession

pub struct PostBoxSession<SM: Message, RM: Message> {
    pub postbox: PostBox<SM, RM>,
    pub kind: SessionKind,
}

// PostBox

pub struct PostBox<SM: Message, RM: Message> {
    uid: u64,
    // The recv end for the incoming mpsc
    recv: mpsc::Receiver<RM>,
    // The send end for the PostOffice outgoing mpsc
    po_send: mpsc::Sender<Letter<SM>>,
}

impl<SM: Message, RM: Message> PostBox<SM, RM> {
    pub fn send(&self, msg: SM) -> Result<(), SendError<Letter<SM>>> {
        self.po_send.send(Letter::Message {
            uid: self.uid,
            payload: msg,
        })
    }

    pub fn recv(&self) -> Result<RM, RecvError> { self.recv.recv() }
    pub fn close(self) -> Result<(), SendError<Letter<SM>>> {
        self.po_send.send(Letter::CloseBox(self.uid))
    }
}

impl<SM: Message, RM: Message> Drop for PostBox<SM, RM> {
    fn drop(&mut self) {
        let _ = self.po_send.send(Letter::CloseBox(self.uid));
    }
}

// PostOffice

pub struct PostOffice<SM: Message, RM: Message> {
    uid_counter: AtomicU64,

    // The send/recv ends for the outgoing mpsc
    recv: Mutex<mpsc::Receiver<Letter<SM>>>,
    send_tmp: Mutex<mpsc::Sender<Letter<SM>>>,

    // The send ends for the PostBox incoming mpscs
    pb_sends: Mutex<HashMap<u64, mpsc::Sender<RM>>>,

    // Internal connection used for networking
    conn: Arc<Connection<Letter<RM>>>,
}

impl<SM: Message, RM: Message> PostOffice<SM, RM> {
    // Create a postoffice that runs on the client, talking to a server
    pub fn to_server<U: ToSocketAddrs>(remote_addr: U) -> Result<Arc<PostOffice<SM, RM>>, Error> {
        // Client-side UIDs start from 1 and count odds
        Ok(PostOffice::new_internal(1, Connection::new(&remote_addr, UdpMgr::new())?))
    }

    // Create a postoffice that runs on the server, talking to a client
    pub fn to_client(stream: TcpStream) -> Result<Arc<PostOffice<SM, RM>>, Error> {
        // Server-side UIDs start from 0 and count evens
        Ok(PostOffice::new_internal(0, Connection::new_stream(stream, UdpMgr::new())?))
    }

    // Create a postoffice with a few characteristics
    pub fn new_internal(start_uid: u64, conn: Arc<Connection<Letter<RM>>>) -> Arc<PostOffice<SM, RM>> {
        // Start the internal connection
        Connection::start(&conn);

        // Create the mpsc for outgoing messages
        let (send_tmp, recv) = mpsc::channel();
        let (send_tmp, recv) = (Mutex::new(send_tmp), Mutex::new(recv));

        Arc::new(PostOffice {
            uid_counter: AtomicU64::new(start_uid),
            recv,
            send_tmp,
            pb_sends: Mutex::new(HashMap::new()),
            conn,
        })
    }

    // Start the worker thread that sends outgoing messages using the Connection instance
    pub fn start(po: Arc<PostOffice<SM, RM>>) {
        thread::spawn(move || {
            let recv = po.recv.lock().unwrap();
            while let Ok(letter) = recv.recv() {
                po.conn.send(letter);
            }
        });
    }

    // Utility to generate a new postbox UID. Server uses even integers, client uses odd integers.
    fn generate_uid(&self) -> u64 { self.uid_counter.fetch_add(2, Ordering::Relaxed) }

    // Utility to create a new postbox with a predetermined UID (not visible to the user)
    fn create_postbox_with_uid(&self, uid: u64) -> PostBox<SM, RM> {
        let (pb_send, pb_recv) = mpsc::channel();
        self.pb_sends.lock().unwrap().insert(uid, pb_send);

        PostBox {
            uid,
            recv: pb_recv,
            po_send: self.send_tmp.lock().unwrap().clone(),
        }
    }

    // Create a new master postbox, triggering the creation of a slave postbox on the other end
    pub fn create_postbox(&self, kind: SessionKind) -> PostBox<SM, RM> {
        let uid = self.generate_uid();
        self.conn.send(Letter::OpenBox::<SM> { uid, kind });
        self.create_postbox_with_uid(uid)
    }

    // Handle incoming packets, returning any new incoming postboxes as they get created
    pub fn await_incoming(&self) -> Result<PostBoxSession<SM, RM>, RecvError> {
        // Keep receiving messages, relaying the messages to their corresponding target postbox.
        // If Letter::OpenBox or Letter::Message are received, handle them appropriately
        loop {
            match self.conn.recv() {
                Ok(Letter::OpenBox { uid, kind }) => {
                    return Ok(PostBoxSession {
                        postbox: self.create_postbox_with_uid(uid),
                        kind,
                    })
                },
                Ok(Letter::CloseBox(uid)) => {
                    self.pb_sends.lock().unwrap().remove(&uid);
                },
                Ok(Letter::Message { uid, payload }) => {
                    self.pb_sends.lock().unwrap().get(&uid).map(|s| s.send(payload));
                },
                Err(e) => return Err(e),
            }
        }
    }
}

impl<SM: Message, RM: Message> Drop for PostOffice<SM, RM> {
    fn drop(&mut self) {
        Connection::stop(&self.conn);
    }
}
