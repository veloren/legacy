use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use terrain::{chunk::Chunk, Container};

pub struct ChunkContainer<P> {
    data: RwLock<Chunk>,
    payload: RwLock<Option<P>>,
}

impl<P> ChunkContainer<P> {
    pub fn new(chunk: Chunk) -> Self {
        ChunkContainer {
            data: RwLock::new(chunk),
            payload: RwLock::new(None),
        }
    }
}

impl<P> Container for ChunkContainer<P> {
    type Payload = P;
    type Cluster = Chunk;

    fn payload(&self) -> RwLockReadGuard<Option<Self::Payload>> { self.payload.read() }
    fn payload_mut(&self) -> RwLockWriteGuard<Option<Self::Payload>> { self.payload.write() }
    fn payload_try(&self) -> Option<RwLockReadGuard<Option<Self::Payload>>> { self.payload.try_read() }
    fn payload_try_mut(&self) -> Option<RwLockWriteGuard<Option<Self::Payload>>> { self.payload.try_write() }
    fn data(&self) -> RwLockReadGuard<Self::Cluster> { self.data.read() }
    fn data_mut(&self) -> RwLockWriteGuard<Self::Cluster> { self.data.write() }
    fn data_try(&self) -> Option<RwLockReadGuard<Self::Cluster>> { self.data.try_read() }
    fn data_try_mut(&self) -> Option<RwLockWriteGuard<Self::Cluster>> { self.data.try_write() }
}
