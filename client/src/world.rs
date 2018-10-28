// Standard
use std::{fs::File, io::prelude::*, path::Path, u8, sync::Arc};

// Library
use vek::*;

// Project
use common::{
    terrain::{
        chunk::{HeterogeneousData, ChunkContainer, HomogeneousData, RleData, Chunk},
        BlockLoader, Container, Key, VolumeIdxVec, VoxelAbsType, VolumeIdxType, SerializeVolume, VolCluster, PersState,
    },
    terrain,
    util::manager::Manager,
};
use parking_lot::{RwLock, Mutex};

// Local
use Client;
use Payloads;
use CHUNK_SIZE;

pub(crate) fn gen_chunk<P: Send + Sync + 'static>(pos: VolumeIdxVec, con: Arc<Mutex<Option<ChunkContainer<P>>>>) {
    let filename = pos.print() + ".dat";
    let filepath = "./saves/".to_owned() + &(filename);
    let path = Path::new(&filepath);
    'load: {
        if path.exists() {
            let mut datfile = File::open(&filepath).unwrap();
            let mut content = Vec::<u8>::new();
            datfile
                .read_to_end(&mut content)
                .expect(&format!("read of file {} failed", &filepath));
            let state = content.remove(0);

            if state == 1 {
                let vol: Result<HomogeneousData, ()> = SerializeVolume::from_bytes(&content);
                if let Ok(vol) = vol {
                    *con.lock() = Some(ChunkContainer::<P>::new(Chunk::Homo(vol)));
                    break 'load;
                }
            } else {
                let vol: Result<RleData, ()> = SerializeVolume::from_bytes(&content);
                if let Ok(vol) = vol {
                    *con.lock() = Some(ChunkContainer::<P>::new(Chunk::Rle(vol)));
                    break 'load;
                }
            }
        }
        let vol = HeterogeneousData::test(
            terrain::volidx_to_voxabs(pos, Vec3::from_slice(&CHUNK_SIZE)),
            Vec3::from_slice(&CHUNK_SIZE),
        );
        *con.lock() = Some(ChunkContainer::<P>::new(Chunk::Hetero(vol)));
    }
}

pub(crate) fn drop_chunk<P: Send + Sync + 'static>(pos: VolumeIdxVec, con: Arc<ChunkContainer<P>>) {
    let filename = pos.print() + ".dat";
    let filepath = "./saves/".to_owned() + &(filename);
    let path = Path::new(&filepath);
    'load: {
        if !path.exists() {
            let mut content = vec![];
            let mut data = con.data_mut();
            let mut ser = data.prefered_serializeable();
            if ser.is_none() {
                data.convert(PersState::Rle);
                ser = data.prefered_serializeable();
            }
            if let Some(ser) = ser {
                let mut bytes = Vec::<u8>::new();
                if data.contains(PersState::Rle) {
                    bytes.push(2);
                } else {
                    if data.contains(PersState::Homo) {
                        bytes.push(1);
                    } else {
                        panic!("what the heck!, this state wasnt planed!")
                    }
                }
                let to_bytes = ser.to_bytes();
                if let Ok(to_bytes) = to_bytes {
                    bytes.extend(&to_bytes);
                    content.extend_from_slice(&bytes);
                    let mut datfile = File::create(filepath).unwrap();
                    datfile.write_all(&content).unwrap();
                    debug!("write to file: {}, bytes: {}", filename, content.len());
                }
            }
        }
    }
}

impl<P: Payloads> Client<P> {
    pub(crate) fn maintain_chunks(&self, _mgr: &mut Manager<Self>) {
        let vol_size = Vec3::from_slice(&CHUNK_SIZE);
        if let Some(player_entity) = self.player_entity() {
            // Find the chunk the player is in
            let (player_pos, player_vel);
            {
                let player = player_entity.read();
                player_pos = player.pos().map(|e| e as VoxelAbsType);
                player_vel = player.vel().map(|e| e as VoxelAbsType);
            }

            const GENERATION_FACTOR: f32 = 1.4; // generate more than you see
            let view_dist = (self.view_distance as f32 * GENERATION_FACTOR) as VolumeIdxType;
            let view_dist_block = terrain::volidx_to_voxabs(Vec3::new(view_dist, view_dist, view_dist), vol_size);
            let mut bl = self.chunk_mgr().block_loader_mut();
            bl.clear();
            bl.push(Arc::new(RwLock::new(BlockLoader{pos: player_pos, size: view_dist_block}))); //player
            bl.push(Arc::new(RwLock::new(BlockLoader{pos: player_pos + player_vel * 5, size: view_dist_block}))); // player in 5 sec
        }
        self.chunk_mgr().maintain();
        self.chunk_mgr().debug();
    }
}
