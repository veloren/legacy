// Standard
use std::{fs::File, io::prelude::*, path::Path, u8, sync::Arc};

// Library
use vek::*;

// Project
use common::{
    terrain::{
        chunk::{HeterogeneousData, ChunkContainer, HomogeneousData, RleData, Chunk},
        Container, Key, PersState, VolumeIdxVec, VoxelAbsType, VolumeIdxType, SerializeVolume, VolCluster,
    },
    terrain,
    util::manager::Manager,
};
use parking_lot::{Mutex};

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
            let mut datfile = File::open(&filename).unwrap();
            let mut content = Vec::<u8>::new();
            datfile
                .read_to_end(&mut content)
                .expect(&format!("read of file {} failed", &filename));
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
            terrain::volidx_to_voxabs(pos, Vec3::new(CHUNK_SIZE[0], CHUNK_SIZE[1], CHUNK_SIZE[2])),
            Vec3::from_slice(&CHUNK_SIZE),
        );
        *con.lock() = Some(ChunkContainer::<P>::new(Chunk::Hetero(vol)));
    }
}

pub(crate) fn drop_chunk<P: Send + Sync + 'static>(pos: VolumeIdxVec, con: Arc<Mutex<Option<ChunkContainer<P>>>>) {

}

impl<P: Payloads> Client<P> {
    pub(crate) fn load_unload_chunks(&self, mgr: &mut Manager<Self>) {
        self.chunk_mgr().maintain();

        // Only update chunks if the player exists
        if let Some(player_entity) = self.player_entity() {
            // Find the chunk the player is in
            let player_pos = player_entity.read().pos().map(|e| e as VoxelAbsType);
            let player_chunk = terrain::voxabs_to_volidx(player_pos, Vec3::new(CHUNK_SIZE[0], CHUNK_SIZE[1], CHUNK_SIZE[2]));

            // Collect chunks around the player
            const GENERATION_FACTOR: f32 = 1.4;
            let mut chunks = vec![];
            let view_dist = (self.view_distance as f32 * GENERATION_FACTOR) as VolumeIdxType;
            for i in player_chunk.x - view_dist..player_chunk.x + view_dist + 1 {
                for j in player_chunk.y - view_dist..player_chunk.y + view_dist + 1 {
                    for k in player_chunk.z - view_dist..player_chunk.z + view_dist + 1 {
                        let pos = Vec3::new(i, j, k);
                        let diff = (player_chunk - pos).map(|e| e.abs()).sum();
                        chunks.push((diff, pos));
                    }
                }
            }
            chunks.sort_by(|a, b| a.0.cmp(&b.0));

            // Generate chunks around the player
            const MAX_CHUNKS_IN_QUEUE: usize = 12; // to not overkill the vol_mgr
            for (_diff, pos) in chunks.iter() {
                if !self.chunk_mgr().exists_chunk(*pos) {
                    // generate up to MAX_CHUNKS_IN_QUEUE chunks around the player
                    if self.chunk_mgr().pending_chunk_cnt() < MAX_CHUNKS_IN_QUEUE {
                        self.chunk_mgr().gen(*pos);
                    }
                }
            }

            const DIFF_TILL_UNLOAD: VolumeIdxType = 5;
            //unload chunks that have a distance of 5 or greater that the last rendered chunk, so that we dont unload to fast, e.g. if we go back a chunk
            let unload_chunk_diff = chunks.last().unwrap().0 + DIFF_TILL_UNLOAD;

            //drop old chunks
            {
                /*
                let chunks = self.chunk_mgr().persistence().hot();
                for (pos, container) in chunks.iter() {
                    let diff = (player_chunk - *pos).map(|e| e.abs()).sum();
                    if diff > unload_chunk_diff {
                        let mut lock = container.vols_mut();
                        let state;
                        if lock.contains(PersState::Homo) {
                            state = PersState::Homo;
                        } else {
                            if !lock.contains(PersState::Rle) {
                                lock.convert(PersState::Rle);
                            }
                            state = PersState::Rle;
                        }
                        let filename = pos.print() + ".dat";
                        let filepath = "./saves/".to_owned() + &(filename);
                        let mut content = vec![]; /*magic number*/
                        if state == PersState::Homo {
                            // This is serialization of PersState, omg, so bad coding. Hate myself for this
                            content.push(1);
                        } else {
                            content.push(2);
                        }
                        let ser = lock.get_serializeable(state);
                        content.extend_from_slice(&ser.to_bytes());
                        let mut datfile = File::create(filepath).unwrap();
                        datfile.write_all(&content).unwrap();
                        debug!("write to file: {}, bytes: {}", filename, content.len());


                        *lock.payload_mut() = None;
                        *lock.remove(state);
                    }
                }
                */
            }
        }
    }
}
