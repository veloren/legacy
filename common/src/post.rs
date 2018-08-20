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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Letter<M> {
    OpenBox { uid: u64, kind: SessionKind },
    CloseBox(u64),
    Message { uid: u64, payload: M },
}
impl<M: Message> Message for Letter<M> {}

pub struct PostBoxSession<SM: Message, RM: Message> {
    pub postbox: PostBox<SM, RM>,
    pub kind: SessionKind,
}

pub struct PostBox<SM: Message, RM: Message> {
    uid: u64,
    incoming: mpsc::Receiver<RM>,
    outgoing: mpsc::Sender<Letter<SM>>,
}
impl<SM: Message, RM: Message> PostBox<SM, RM> {
    pub fn send(&self, msg: SM) -> Result<(), SendError<Letter<SM>>> {
        self.outgoing.send(Letter::Message {
            uid: self.uid,
            payload: msg,
        })
    }
    pub fn recv(&self) -> Result<RM, RecvError> { self.incoming.recv() }
    pub fn close(&self) -> Result<(), SendError<Letter<SM>>> { self.outgoing.send(Letter::CloseBox(self.uid)) }
}
impl<SM: Message, RM: Message> Drop for PostBox<SM, RM> {
    fn drop(&mut self) { let _ = self.close(); }
}

pub struct PostOffice<SM: Message, RM: Message> {
    uid_counter: AtomicU64,
    from_postbox: Mutex<mpsc::Receiver<Letter<SM>>>,
    to_postoffice: Mutex<mpsc::Sender<Letter<SM>>>,
    boxes: Mutex<HashMap<u64, mpsc::Sender<RM>>>,
    conn: Arc<Connection<Letter<RM>>>,
}

impl<SM: Message, RM: Message> PostOffice<SM, RM> {
    pub fn new_remote<U: ToSocketAddrs>(remote_addr: U) -> Result<Arc<PostOffice<SM, RM>>, Error> {
        let (there, here) = mpsc::channel();

        let conn = Connection::new::<U>(&remote_addr, UdpMgr::new()).unwrap();
        Connection::start(&conn);

        Ok(Arc::new(PostOffice::new_internal(conn, here, there)))
    }

    pub fn new_host(stream: TcpStream) -> Result<Arc<PostOffice<SM, RM>>, Error> {
        let (there, here) = mpsc::channel();

        let conn = Connection::new_stream(stream, UdpMgr::new()).unwrap();
        Connection::start(&conn);

        Ok(Arc::new(PostOffice::new_internal(conn, here, there)))
    }

    fn new_internal(
        conn: Arc<Connection<Letter<RM>>>,
        from_postbox: mpsc::Receiver<Letter<SM>>,
        to_postoffice: mpsc::Sender<Letter<SM>>,
    ) -> PostOffice<SM, RM> {
        PostOffice {
            uid_counter: AtomicU64::new(0),
            from_postbox: Mutex::new(from_postbox),
            to_postoffice: Mutex::new(to_postoffice),
            boxes: Mutex::new(HashMap::new()),
            conn: conn,
        }
    }

    pub fn start(manager: Arc<PostOffice<SM, RM>>) {
        thread::spawn(move || loop {
            if let Ok(recv) = manager.from_postbox.lock() {
                match recv.recv() {
                    Ok(letter) => manager.conn.send(letter),
                    Err(_) => break,
                };
            }
        });
    }

    pub fn stop(manager: Arc<PostOffice<SM, RM>>) {
        Connection::stop(&manager.conn);
        // non blocking stop for now
    }

    fn generate_uid(&self) -> u64 { self.uid_counter.fetch_add(1, Ordering::Relaxed) }

    fn create_postbox_with_uid(&self, uid: u64) -> PostBox<SM, RM> {
        let (here, there) = mpsc::channel();
        self.boxes.lock().unwrap().insert(uid, here);

        PostBox {
            uid,
            incoming: there,
            outgoing: self.to_postoffice.lock().unwrap().clone(),
        }
    }

    pub fn create_postbox(&self, kind: SessionKind) -> PostBox<SM, RM> {
        let uid = self.generate_uid();
        self.conn.send(Letter::OpenBox::<SM> { uid, kind });
        self.create_postbox_with_uid(uid)
    }

    pub fn await_incoming(&self) -> Result<PostBoxSession<SM, RM>, RecvError> {
        loop {
            match self.conn.recv() {
                Ok(Letter::OpenBox { uid, kind }) => {
                    return Ok(PostBoxSession {
                        postbox: self.create_postbox_with_uid(uid),
                        kind,
                    })
                },
                Ok(Letter::CloseBox(uid)) => {
                    self.boxes.lock().unwrap().remove(&uid);
                },
                Ok(Letter::Message { uid, payload }) => {
                    self.boxes.lock().unwrap().get(&uid).map(|s| s.send(payload));
                },
                // Err(TryRecvError::Empty) => continue,
                Err(e) => return Err(e),
            }
        }
    }
}
