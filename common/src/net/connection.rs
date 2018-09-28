// Standard
use std::{
    collections::{vec_deque::VecDeque, HashMap},
    net::{SocketAddr, TcpStream, ToSocketAddrs, UdpSocket},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, RecvError, TryRecvError},
        Arc,
    },
    thread::{self, JoinHandle},
};

// Library
use get_if_addrs::get_if_addrs;
use parking_lot::{Mutex, MutexGuard, RwLock};

// Parent
use super::{
    packet::{Frame, FrameError, IncomingPacket, OutgoingPacket},
    protocol::Protocol,
    tcp::Tcp,
    udp::Udp,
    udpmgr::UdpMgr,
    ConnectionMessage, Error, Message,
};

#[derive(Debug)]
enum ConnectionError {
    Disconnected,
}

#[derive(Debug)]
pub struct Connection<RM: Message> {
    // sorted by prio and then chronically
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
    recvd_message_write: Mutex<mpsc::Sender<Result<RM, ConnectionError>>>,
    recvd_message_read: Mutex<mpsc::Receiver<Result<RM, ConnectionError>>>,
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

        //let (error_sender, error_receiver) = mpsc::channel();
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
            //error_write: Mutex::new(error_sender),
            //error_read: Mutex::new(error_receiver),
        };

        Ok(Arc::new(m))
    }

    pub fn open_udp<'b>(manager: &'b Arc<Connection<RM>>, listen: SocketAddr, sender: SocketAddr) {
        if let Some(..) = *manager.udp.lock() {
            panic!("not implemented");
        }
        *manager.udp.lock() = Some(Udp::new(listen, sender).unwrap());
        manager.send(ConnectionMessage::OpenedUdp { host: listen });

        let m = manager.clone();
        let mut rt = manager.recv_thread_udp.lock();
        *rt = Some(thread::spawn(move || {
            m.recv_worker_udp();
        }));

        let m = manager.clone();
        let mut st = manager.send_thread_udp.lock();
        *st = Some(thread::spawn(move || {
            m.send_worker_udp();
        }));
    }

    pub fn start<'b>(manager: &'b Arc<Connection<RM>>) {
        let m = manager.clone();
        let mut rt = manager.recv_thread.lock();
        *rt = Some(thread::spawn(move || {
            m.recv_worker();
        }));

        let m = manager.clone();
        let mut st = manager.send_thread.lock();
        *st = Some(thread::spawn(move || {
            m.send_worker();
        }));
    }

    pub fn stop<'b>(manager: &'b Arc<Connection<RM>>) {
        let m = manager.clone();
        m.running.store(false, Ordering::Relaxed);
        m.recvd_message_write.lock().send(Err(ConnectionError::Disconnected));
        // non blocking stop for now
    }

    pub fn send<M: Message>(&self, message: M) {
        let mut id = self.next_id.lock();
        self.packet_out.lock()[16].push_back(OutgoingPacket::new(message.to_bytes().unwrap(), *id));
        *id += 1;
        let mut p = self.packet_out_count.write();
        *p += 1;
        let mut rt = self.send_thread.lock();
        if let Some(cb) = rt.as_mut() {
            //trigger sending
            cb.thread().unpark();
        }
    }

    pub fn try_recv(&self) -> Result<RM, ()> {
        match self.recvd_message_read.lock().try_recv() {
            Ok(Ok(msg)) => return Ok(msg),
            _ => Err(()),
        }
    }

    pub fn recv(&self) -> Result<RM, ()> {
        match self.recvd_message_read.lock().recv() {
            Ok(Ok(msg)) => return Ok(msg),
            _ => return Err(()),
        }

        match self.recvd_message_read.lock().recv() {
            Ok(Ok(msg)) => return Ok(msg),
            Ok(Err(_)) => return Err(()),
            Err(_) => return Err(()),
        }

        return Err(());
    }

    fn send_worker(&self) {
        loop {
            if !self.running.load(Ordering::Relaxed) {
                break;
            }
            if *self.packet_out_count.read() == 0 {
                thread::park();
                continue;
            }
            // find next package
            let mut packets = self.packet_out.lock();
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
                            let mut p = self.packet_out_count.write();
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
                            let mut packets = self.packet_in.lock();
                            packets.insert(id, msg);
                        },
                        Frame::Data { id, .. } => {
                            let mut packets = self.packet_in.lock();
                            let packet = packets.get_mut(&id);
                            if packet.unwrap().load_data_frame(frame) {
                                //convert
                                let packet = packets.get_mut(&id);
                                let data = packet.unwrap().data();
                                debug!("received packet: {:?}", &data);

                                let recvd_message_write = self.recvd_message_write.lock();
                                recvd_message_write.send(Ok(RM::from_bytes(data).unwrap())).unwrap();
                            }
                        },
                    }
                },
                Err(e) => {
                    error!("Net Error {:?}", &e);

                    // TODO: Handle errors that can be resolved locally
                    match e {
                        _ => {
                            let recvd_message_write = self.recvd_message_write.lock();
                            recvd_message_write.send(Err(ConnectionError::Disconnected)).unwrap();
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
            if *self.packet_out_count.read() == 0 {
                thread::park();
                continue;
            }
            // find next package
            let mut packets = self.packet_out.lock();
            for i in 0..255 {
                if packets[i].len() != 0 {
                    // build part
                    const SPLIT_SIZE: u64 = 2000;
                    match packets[i][0].generate_frame(SPLIT_SIZE) {
                        Ok(frame) => {
                            // send it
                            let mut udp = self.udp.lock();
                            udp.as_mut().unwrap().send(frame).unwrap();
                        },
                        Err(FrameError::SendDone) => {
                            packets[i].pop_front();
                            let mut p = self.packet_out_count.write();
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
            let mut udp = self.udp.lock();
            let frame = udp.as_mut().unwrap().recv();
            match frame {
                Ok(frame) => {
                    match frame {
                        Frame::Header { id, .. } => {
                            let msg = IncomingPacket::new(frame);
                            let mut packets = self.packet_in.lock();
                            packets.insert(id, msg);
                        },
                        Frame::Data { id, .. } => {
                            let mut packets = self.packet_in.lock();
                            let packet = packets.get_mut(&id);
                            if packet.unwrap().load_data_frame(frame) {
                                //convert
                                let packet = packets.get_mut(&id);
                                let data = packet.unwrap().data();
                                debug!("received packet: {:?}", &data);

                                let recvd_message_write = self.recvd_message_write.lock();
                                recvd_message_write.send(Ok(RM::from_bytes(data).unwrap())).unwrap();
                            }
                        },
                    }
                },
                Err(e) => {
                    error!("Net Error {:?}", &e);

                    // TODO: Handle errors that can be resolved locally
                    match e {
                        _ => {
                            let recvd_message_write = self.recvd_message_write.lock();
                            recvd_message_write.send(Err(ConnectionError::Disconnected)).unwrap();
                        },
                    }
                },
            }
        }
    }

    #[allow(dead_code)]
    fn bind_udp<T: ToSocketAddrs>(bind_addr: &T) -> Result<UdpSocket, Error> {
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
