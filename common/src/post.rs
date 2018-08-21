// Standard
use std::{
    collections::HashMap,
    net::{TcpStream, ToSocketAddrs},
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        mpsc::{self, RecvError, SendError},
        Arc, Mutex,
    },
    thread,
};

// Local
use net::{Connection, Error, Message, UdpMgr};

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
pub enum Letter<SK, M> {
    OpenBox { uid: u64, kind: SK },
    CloseBox(u64),
    Message { uid: u64, payload: M },
}

impl<SK: Message, M: Message> Message for Letter<SK, M> {}

// PostBoxSession

pub struct PostBoxSession<SK: Message, SM: Message, RM: Message> {
    pub postbox: PostBox<SK, SM, RM>,
    pub kind: SK,
}

// PostBox

pub struct PostBox<SK: Message, SM: Message, RM: Message> {
    uid: u64,
    // The recv end for the incoming mpsc
    recv: mpsc::Receiver<RM>,
    // The send end for the PostOffice outgoing mpsc
    po_send: mpsc::Sender<Letter<SK, SM>>,
}

impl<SK: Message, SM: Message, RM: Message> PostBox<SK, SM, RM> {
    pub fn send(&self, msg: SM) -> Result<(), SendError<Letter<SK, SM>>> {
        self.po_send.send(Letter::Message {
            uid: self.uid,
            payload: msg,
        })
    }

    pub fn recv(&self) -> Result<RM, RecvError> { self.recv.recv() }

    pub fn close(self) -> Result<(), SendError<Letter<SK, SM>>> { self.po_send.send(Letter::CloseBox(self.uid)) }
}

impl<SK: Message, SM: Message, RM: Message> Drop for PostBox<SK, SM, RM> {
    fn drop(&mut self) { let _ = self.po_send.send(Letter::CloseBox(self.uid)); }
}

// PostOffice

pub struct PostOffice<SK: Message, SM: Message, RM: Message> {
    uid_counter: AtomicU64,

    // The send end of the outgoing mpsc, used for cloning and passing to postboxes
    send_tmp: Mutex<mpsc::Sender<Letter<SK, SM>>>,

    // The send ends for the PostBox incoming mpscs
    pb_sends: Mutex<HashMap<u64, mpsc::Sender<RM>>>,

    // Data shared with worker thread: running bool and internal connection used for networking
    running: Arc<AtomicBool>,
    conn: Arc<Connection<Letter<SK, RM>>>,
    worker: Option<thread::JoinHandle<()>>,
}

impl<SK: Message, SM: Message, RM: Message> PostOffice<SK, SM, RM> {
    // Create a postoffice that runs on the client, talking to a server
    pub fn to_server<U: ToSocketAddrs>(remote_addr: U) -> Result<PostOffice<SK, SM, RM>, Error> {
        // Client-side UIDs start from 1 and count odds
        Ok(PostOffice::new_internal(
            1,
            Connection::new(&remote_addr, UdpMgr::new())?,
        ))
    }

    // Create a postoffice that runs on the server, talking to a client
    pub fn to_client(stream: TcpStream) -> Result<PostOffice<SK, SM, RM>, Error> {
        // Server-side UIDs start from 0 and count evens
        Ok(PostOffice::new_internal(
            0,
            Connection::new_stream(stream, UdpMgr::new())?,
        ))
    }

    // Create a postoffice with a few characteristics
    pub fn new_internal(start_uid: u64, conn: Arc<Connection<Letter<SK, RM>>>) -> PostOffice<SK, SM, RM> {
        // Start the internal connection
        Connection::start(&conn);

        // Create the mpsc for outgoing messages
        let (send_tmp, recv) = mpsc::channel();
        let send_tmp = Mutex::new(send_tmp);

        // Create shared running flag
        let running = Arc::new(AtomicBool::new(true));

        // Start the worker thread that sends outgoing messages using the Connection instance
        let running_ref = running.clone();
        let conn_ref = conn.clone();
        let worker = Some(thread::spawn(move || {
            while running_ref.load(Ordering::Relaxed) {
                match recv.recv() {
                    Ok(letter) => conn_ref.send(letter),
                    Err(_) => break,
                }
            }

            Connection::stop(&conn_ref);
        }));

        PostOffice {
            uid_counter: AtomicU64::new(start_uid),
            send_tmp,
            pb_sends: Mutex::new(HashMap::new()),
            running,
            conn,
            worker,
        }
    }

    // Utility to generate a new postbox UID. Server uses even integers, client uses odd integers.
    fn generate_uid(&self) -> u64 { self.uid_counter.fetch_add(2, Ordering::Relaxed) }

    // Utility to create a new postbox with a predetermined UID (not visible to the user)
    fn create_postbox_with_uid(&self, uid: u64) -> PostBox<SK, SM, RM> {
        let (pb_send, pb_recv) = mpsc::channel();
        self.pb_sends.lock().unwrap().insert(uid, pb_send);

        PostBox {
            uid,
            recv: pb_recv,
            po_send: self.send_tmp.lock().unwrap().clone(),
        }
    }

    // Create a new master postbox, triggering the creation of a slave postbox on the other end
    pub fn create_postbox(&self, kind: SK) -> PostBox<SK, SM, RM> {
        let uid = self.generate_uid();
        self.conn.send(Letter::OpenBox::<SK, SM> { uid, kind });
        self.create_postbox_with_uid(uid)
    }

    // Handle incoming packets, returning any new incoming postboxes as they get created
    pub fn await_incoming(&self) -> Result<PostBoxSession<SK, SM, RM>, RecvError> {
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

impl<SK: Message, SM: Message, RM: Message> Drop for PostOffice<SK, SM, RM> {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        self.worker.take().map(|w| w.join());
    }
}
