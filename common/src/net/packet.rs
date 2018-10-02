#[derive(Debug)]
pub enum Frame {
    Header { id: u64, length: u64 },
    Data { id: u64, frame_no: u64, data: Vec<u8> },
}

#[derive(Debug)]
pub enum FrameError {
    SendDone,
}

//TODO: enhance this PacketData / OutgoingPacket structure, so that only one byte stream is keept for broadcast
#[derive(Debug)]
pub struct PacketData {
    bytes: Vec<u8>,
    id: u64,
}

#[derive(Debug)]
pub struct OutgoingPacket {
    data: PacketData,
    pos: u64,
    headersend: bool,
    dataframesno: u64,
    prio: u8,
}

#[derive(Debug)]
pub struct IncomingPacket {
    data: PacketData,
    pos: u64,
    dataframesno: u64,
}

impl PacketData {
    pub fn new(bytes: Vec<u8>, id: u64) -> PacketData { PacketData { bytes, id } }

    pub fn new_size(size: u64, id: u64) -> PacketData {
        PacketData {
            bytes: vec![0; size as usize],
            id,
        }
    }
}

impl OutgoingPacket {
    pub fn new(bytes: Vec<u8>, id: u64) -> OutgoingPacket {
        OutgoingPacket {
            data: PacketData::new(bytes, id),
            pos: 0,
            headersend: false,
            dataframesno: 0,
            prio: 16,
        }
    }

    // maximal size of the frame (implementation aprox)
    pub fn generate_frame(&mut self, size: u64) -> Result<Frame, FrameError> {
        if !self.headersend {
            self.headersend = true;
            Ok(Frame::Header {
                id: self.data.id,
                length: self.data.bytes.len() as u64,
            })
        } else {
            let remaining = self.data.bytes.len() as u64 - self.pos;
            if remaining == 0 {
                return Err(FrameError::SendDone);
            }
            let to_send = if size >= remaining { remaining } else { size };
            let end_pos = self.pos + to_send;
            let frame = Frame::Data {
                id: self.data.id,
                frame_no: self.dataframesno,
                data: self.data.bytes[self.pos as usize..end_pos as usize].to_vec(),
            };
            self.pos += to_send as u64;
            self.dataframesno += 1;
            Ok(frame)
        }
    }

    #[allow(dead_code)]
    pub fn prio(&self) -> &u8 { &self.prio }
}

impl IncomingPacket {
    pub fn new(header: Frame) -> IncomingPacket {
        match header {
            Frame::Header { id, length } => IncomingPacket {
                data: PacketData::new_size(length, id),
                pos: 0,
                dataframesno: 0,
            },
            Frame::Data { .. } => {
                panic!("not implemented");
            },
        }
    }

    // returns finished
    pub fn load_data_frame(&mut self, data: Frame) -> bool {
        match data {
            Frame::Header { .. } => {
                panic!("not implemented");
            },
            Frame::Data { id, frame_no, data } => {
                if id != self.data.id {
                    panic!("id missmatch {} <> {}", id, self.data.id);
                }
                if frame_no != self.dataframesno {
                    panic!("bufferin for frames not yet implemented");
                }
                // copy data starting from self.pos
                //TODO: check size of send with reserved
                for (dst, src) in self.data.bytes[self.pos as usize..].iter_mut().zip(&data) {
                    *dst = *src;
                }
                self.pos += data.len() as u64;
                self.dataframesno += 1;

                self.pos == self.data.bytes.len() as u64
            },
        }
    }

    #[allow(dead_code)]
    pub fn data(&self) -> &Vec<u8> { &self.data.bytes }
}
