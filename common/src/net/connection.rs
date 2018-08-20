// Standard
use std::{
    collections::{vec_deque::VecDeque, HashMap},
    net::{SocketAddr, TcpStream, ToSocketAddrs, UdpSocket},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, RecvError, TryRecvError},
        Arc, Mutex, RwLock,
    },
    thread::{self, JoinHandle},
};

// Library
use get_if_addrs::get_if_addrs;

// Parent
use super::{
    packet::{Frame, FrameError, IncomingPacket, OutgoingPacket},
    protocol::Protocol,
    tcp::Tcp,
    udp::Udp,
    udpmgr::UdpMgr,
    ConnectionMessage, Error, Message,
};

enum ConnectionError {
    Disconnected,
}

pub struct Connection<RM: Message> {
    // sorted by prio and then cronically
    tcp: Tcp,
    udpmgr: Arc<UdpMgr>,
    udp: Mutex<Option<Udp>>,
    packet_in: Mutex<HashMap<u64, IncomingPacket>>,
    packet_out: Mutex<Vec<VecDeque<OutgoingPacket>>>,
    packet_out_count: RwLock<u64>,
    running: AtomicBool,
    send_thread: Mutex<Option<JoinHandle<()>>>,
    recv_thread: Mutex<Option<JoinHandle<()>>>,
    send_thread_udp: Mutex<Option<JoinHandle<()>>>,
    recv_thread_udp: Mutex<Option<JoinHandle<()>>>,
    next_id: Mutex<u64>,

    // Message channel
    recvd_message_write: Mutex<mpsc::Sender<RM>>,
    recvd_message_read: Mutex<mpsc::Receiver<RM>>,

    // Error channel
    error_write: Mutex<mpsc::Sender<ConnectionError>>,
    error_read: Mutex<mpsc::Receiver<ConnectionError>>,
}

impl<RM: Message> Connection<RM> {
    pub fn new<A: ToSocketAddrs>(remote: &A, udpmgr: Arc<UdpMgr>) -> Result<Arc<Connection<RM>>, Error> {
        Connection::new_internal(Tcp::new(&remote)?, udpmgr)
    }

    pub fn new_stream(stream: TcpStream, udpmgr: Arc<UdpMgr>) -> Result<Arc<Connection<RM>>, Error> {
        Connection::new_internal(Tcp::new_stream(stream)?, udpmgr)
    }

    fn new_internal(tcp: Tcp, udpmgr: Arc<UdpMgr>) -> Result<Arc<Connection<RM>>, Error> {
        let mut packet_out = Vec::new();
        for _i in 0..255 {
            packet_out.push(VecDeque::new());
        }

        let (error_sender, error_receiver) = mpsc::channel();
        let (message_sender, message_receiver) = mpsc::channel();

        let m = Connection {
            tcp,
            udpmgr,
            udp: Mutex::new(None),
            packet_in: Mutex::new(HashMap::new()),
            packet_out_count: RwLock::new(0),
            packet_out: Mutex::new(packet_out),
            running: AtomicBool::new(true),
            send_thread: Mutex::new(None),
            recv_thread: Mutex::new(None),
            send_thread_udp: Mutex::new(None),
            recv_thread_udp: Mutex::new(None),
            next_id: Mutex::new(1),
            recvd_message_write: Mutex::new(message_sender),
            recvd_message_read: Mutex::new(message_receiver),
            error_write: Mutex::new(error_sender),
            error_read: Mutex::new(error_receiver),
        };

        Ok(Arc::new(m))
    }

    pub fn open_udp<'b>(manager: &'b Arc<Connection<ConnectionMessage>>, listen: SocketAddr, sender: SocketAddr) {
        if let Some(..) = *manager.udp.lock().unwrap() {
            panic!("not implemented");
        }

        let msg = ConnectionMessage::OpenedUdp { host: listen };

        *manager.udp.lock().unwrap() = Some(Udp::new(listen, sender).unwrap());
        manager.send(msg);

        let m = manager.clone();
        let mut rt = manager.recv_thread_udp.lock().unwrap();
        *rt = Some(thread::spawn(move || {
            m.recv_worker_udp();
        }));

        let m = manager.clone();
        let mut st = manager.send_thread_udp.lock().unwrap();
        *st = Some(thread::spawn(move || {
            m.send_worker_udp();
        }));
    }

    pub fn start<'b>(manager: &'b Arc<Connection<RM>>) {
        let m = manager.clone();
        let mut rt = manager.recv_thread.lock().unwrap();
        *rt = Some(thread::spawn(move || {
            m.recv_worker();
        }));

        let m = manager.clone();
        let mut st = manager.send_thread.lock().unwrap();
        *st = Some(thread::spawn(move || {
            m.send_worker();
        }));
    }

    pub fn stop<'b>(manager: &'b Arc<Connection<RM>>) {
        let m = manager.clone();
        m.running.store(false, Ordering::Relaxed);
        // non blocking stop for now
    }

    pub fn send<M: Message>(&self, message: M) {
        let mut id = self.next_id.lock().unwrap();
        self.packet_out.lock().unwrap()[16].push_back(OutgoingPacket::new(message.to_bytes().unwrap(), *id));
        *id += 1;
        let mut p = self.packet_out_count.write().unwrap();
        *p += 1;
        let rt = self.send_thread.lock();
        if let Some(cb) = rt.unwrap().as_mut() {
            //trigger sending
            cb.thread().unpark();
        }
    }

    pub fn try_recv(&self) -> Result<RM, TryRecvError> {
        if let Ok(error_read) = self.error_read.lock() {
            match error_read.try_recv() {
                Ok(ConnectionError::Disconnected) => return Err(TryRecvError::Disconnected),
                Err(TryRecvError::Disconnected) => return Err(TryRecvError::Disconnected),
                _ => {},
            }
        }

        if let Ok(recvd_message_read) = self.recvd_message_read.lock() {
            match recvd_message_read.try_recv() {
                Ok(msg) => return Ok(msg),
                Err(e) => return Err(e),
            }
        }

        return Err(TryRecvError::Disconnected);
    }

    pub fn recv(&self) -> Result<RM, RecvError> {
        if let Ok(error_read) = self.error_read.lock() {
            match error_read.try_recv() {
                Ok(ConnectionError::Disconnected) => return Err(RecvError),
                Err(TryRecvError::Disconnected) => return Err(RecvError),
                _ => {},
            }
        }

        if let Ok(recvd_message_read) = self.recvd_message_read.lock() {
            match recvd_message_read.recv() {
                Ok(msg) => return Ok(msg),
                Err(e) => return Err(e),
            }
        }

        return Err(RecvError);
    }

    fn send_worker(&self) {
        loop {
            if !self.running.load(Ordering::Relaxed) {
                break;
            }
            if *self.packet_out_count.read().unwrap() == 0 {
                thread::park();
                continue;
            }
            // find next package
            let mut packets = self.packet_out.lock().unwrap();
            for i in 0..255 {
                if packets[i].len() != 0 {
                    // build part
                    const SPLIT_SIZE: u64 = 2000;
                    match packets[i][0].generate_frame(SPLIT_SIZE) {
                        Ok(frame) => {
                            // send it
                            self.tcp.send(frame).unwrap();
                        },
                        Err(FrameError::SendDone) => {
                            packets[i].pop_front();
                            let mut p = self.packet_out_count.write().unwrap();
                            *p -= 1;
                        },
                    }

                    break;
                }
            }
        }
    }

    fn recv_worker(&self) {
        loop {
            if !self.running.load(Ordering::Relaxed) {
                break;
            }
            let frame = self.tcp.recv();
            match frame {
                Ok(frame) => {
                    match frame {
                        Frame::Header { id, .. } => {
                            let msg = IncomingPacket::new(frame);
                            let mut packets = self.packet_in.lock().unwrap();
                            packets.insert(id, msg);
                        },
                        Frame::Data { id, .. } => {
                            let mut packets = self.packet_in.lock().unwrap();
                            let packet = packets.get_mut(&id);
                            if packet.unwrap().load_data_frame(frame) {
                                //convert
                                let packet = packets.get_mut(&id);
                                let data = packet.unwrap().data();
                                debug!("received packet: {:?}", &data);

                                let recvd_message_write = self.recvd_message_write.lock().unwrap();
                                recvd_message_write.send(RM::from_bytes(data).unwrap()).unwrap();
                            }
                        },
                    }
                },
                Err(e) => {
                    error!("Net Error {:?}", &e);

                    // TODO: Handle errors that can be resolved locally
                    match e {
                        _ => {
                            if let Ok(error_write) = self.error_write.lock() {
                                error_write.send(ConnectionError::Disconnected).unwrap();
                            }
                        },
                    }
                },
            }
        }
    }

    fn send_worker_udp(&self) {
        loop {
            if !self.running.load(Ordering::Relaxed) {
                break;
            }
            if *self.packet_out_count.read().unwrap() == 0 {
                thread::park();
                continue;
            }
            // find next package
            let mut packets = self.packet_out.lock().unwrap();
            for i in 0..255 {
                if packets[i].len() != 0 {
                    // build part
                    const SPLIT_SIZE: u64 = 2000;
                    match packets[i][0].generate_frame(SPLIT_SIZE) {
                        Ok(frame) => {
                            // send it
                            let mut udp = self.udp.lock().unwrap();
                            udp.as_mut().unwrap().send(frame).unwrap();
                        },
                        Err(FrameError::SendDone) => {
                            packets[i].pop_front();
                            let mut p = self.packet_out_count.write().unwrap();
                            *p -= 1;
                        },
                    }

                    break;
                }
            }
        }
    }

    fn recv_worker_udp(&self) {
        loop {
            if !self.running.load(Ordering::Relaxed) {
                break;
            }
            let mut udp = self.udp.lock().unwrap();
            let frame = udp.as_mut().unwrap().recv();
            match frame {
                Ok(frame) => {
                    match frame {
                        Frame::Header { id, .. } => {
                            let msg = IncomingPacket::new(frame);
                            let mut packets = self.packet_in.lock().unwrap();
                            packets.insert(id, msg);
                        },
                        Frame::Data { id, .. } => {
                            let mut packets = self.packet_in.lock().unwrap();
                            let packet = packets.get_mut(&id);
                            if packet.unwrap().load_data_frame(frame) {
                                //convert
                                let packet = packets.get_mut(&id);
                                let data = packet.unwrap().data();
                                debug!("received packet: {:?}", &data);

                                let recvd_message_write = self.recvd_message_write.lock().unwrap();
                                recvd_message_write.send(RM::from_bytes(data).unwrap()).unwrap();
                            }
                        },
                    }
                },
                Err(e) => {
                    error!("Net Error {:?}", &e);

                    // TODO: Handle errors that can be resolved locally
                    match e {
                        _ => {
                            if let Ok(error_write) = self.error_write.lock() {
                                error_write.send(ConnectionError::Disconnected).unwrap();
                            }
                        },
                    }
                },
            }
        }
    }

    fn bind_udp<U: ToSocketAddrs>(bind_addr: &U) -> Result<UdpSocket, Error> {
        let sock = UdpSocket::bind(&bind_addr);
        match sock {
            Ok(s) => Ok(s),
            Err(_e) => {
                let new_bind = bind_addr.to_socket_addrs()?.next().unwrap().port() + 1;
                let ip = get_if_addrs().unwrap()[0].ip();
                let new_addr = SocketAddr::new(ip, new_bind);
                warn!("Binding local port failed, trying {}", new_addr);
                Connection::<RM>::bind_udp(&new_addr)
            },
        }
    }
}
