// Standard
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};

// Library
use parking_lot::RwLock;

// Local
use audio::{audio_gen::AudioGen, Buffer, Stream};

pub struct AudioMgr<G: AudioGen> {
    //pending: Arc<RwLock<HashMap<Vec3<VolOffs>, Arc<Mutex<Option<ChunkContainer<P>>>>>>>, // Mutex is only needed for compiler, we dont acces it in multiple threads
    streams: RwLock<HashMap<u64, Stream>>,
    buffers: RwLock<HashMap<u64, Buffer>>,
    next_stream_id: AtomicUsize,
    next_buffer_id: AtomicUsize,
    gen: Arc<G>,
}

impl<G: AudioGen> AudioMgr<G> {
    pub fn new(gen: Arc<G>) -> AudioMgr<G> {
        AudioMgr {
            streams: RwLock::new(HashMap::new()),
            buffers: RwLock::new(HashMap::new()),
            next_stream_id: AtomicUsize::new(0),
            next_buffer_id: AtomicUsize::new(0),
            gen,
        }
    }

    pub fn gen_stream(&self, stream: Stream) -> Option<u64> {
        let mut slock = self.streams.write();
        let lock = self.buffers.read();
        let buf = lock.get(&stream.buffer);
        if let Some(buf) = buf {
            let id = self.next_stream_id.fetch_add(1, Ordering::Relaxed) as u64;
            let p = self.gen.gen_stream(id, &buf, &stream);
            slock.insert(id, stream);
            return Some(id);
        }
        None
    }

    pub fn gen_buffer(&self, buffer: Buffer) -> Option<u64> {
        let id = self.next_stream_id.fetch_add(1, Ordering::Relaxed) as u64;
        let p = self.gen.gen_buffer(id, &buffer);
        self.buffers.write().insert(id, buffer);
        Some(id)
    }

    pub fn stream(&self, id: u64) -> Option<Stream> {
        let lock = self.streams.read();
        if let Some(s) = lock.get(&id) {
            return Some(s.clone());
        }
        None
    }

    pub fn buffer(&self, id: u64) -> Option<Buffer> {
        let lock = self.buffers.read();
        if let Some(b) = lock.get(&id) {
            return Some(b.clone());
        }
        None
    }

    pub fn set_stream(&self, id: u64, stream: Stream) {
        let mut lock = self.streams.write();
        if let Some(s) = lock.get_mut(&id) {
            *s = stream;
        }
    }

    pub fn set_buffer(&self, id: u64, buffer: Buffer) {
        let mut lock = self.buffers.write();
        if let Some(b) = lock.get_mut(&id) {
            *b = buffer;
        }
    }

    // regually call this to handle old streams
    pub fn maintain(&self, tick: Duration) {
        let mut slock = self.streams.write();
        let lock = self.buffers.read();
        slock.retain(|id, mut stream| {
            if stream.start_tick + stream.duration < tick {
                let buf = lock.get(&stream.buffer);
                if let Some(buf) = buf {
                    self.gen.drop_stream(*id, &buf, &stream);
                }
                return false;
            }
            true
        });
    }

    pub fn drop_stream(&self, id: u64) {
        let mut slock = self.streams.read();
        let lock = self.buffers.read();
        let stream = slock.get(&id);
        if let Some(stream) = stream {
            let buf = lock.get(&stream.buffer);
            if let Some(buf) = buf {
                self.gen.drop_stream(id, &buf, &stream);
                self.streams.write().remove(&id);
            }
        }
    }

    pub fn drop_buffer(&self, id: u64) { self.buffers.write().remove(&id); }
}
