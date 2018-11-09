// Standard
use std::{fs::File, io::prelude::*, path::Path, sync::Arc, u8};

// Library
use vek::*;

// Project
use common::{
    terrain::{
        self,
        chunk::{Chunk, ChunkContainer, HeterogeneousData},
        BlockLoader, Container, Key, PersState, VolCluster, VolOffs, VoxAbs,
    },
    util::manager::Manager,
};
use parking_lot::{Mutex, RwLock};

// Local
use Client;
use Payloads;
use CHUNK_SIZE;
use world_crate;

pub(crate) fn gen_chunk<P: Send + Sync + 'static>(pos: Vec3<VolOffs>, con: Arc<Mutex<Option<ChunkContainer<P>>>>) {
    let filename = pos.print() + ".dat";
    let filepath = "./saves/".to_owned() + &(filename);
    let path = Path::new(&filepath);
    'load: {
        if path.exists() {
            let datfile = File::open(&filepath);
            if let Ok(mut datfile) = datfile {
                let mut content = Vec::<u8>::new();
                datfile
                    .read_to_end(&mut content)
                    .expect(&format!("read of file {} failed", &filepath));
                let cc = Chunk::from_bytes(&content);
                if let Ok(cc) = cc {
                    *con.lock() = Some(ChunkContainer::<P>::new(cc));
                    break 'load;
                }
            }
            warn!(
                "there was a problem reading a chunk {} from file, it was newly generated",
                pos
            );
        }
        let c = world_crate::World::gen_chunk(pos.map(|e| e as i32));
        *con.lock() = Some(ChunkContainer::<P>::new(c));
    }
}

pub(crate) fn drop_chunk<P: Send + Sync + 'static>(pos: Vec3<VolOffs>, con: Arc<ChunkContainer<P>>) {
    let filename = pos.print() + ".dat";
    let filepath = "./saves/".to_owned() + &(filename);
    let path = Path::new(&filepath);
    'load: {
        if !path.exists() {
            let mut data = con.data_mut();
            let bytes = data.to_bytes();
            if let Ok(bytes) = bytes {
                let datfile = File::create(filepath);
                if let Ok(mut datfile) = datfile {
                    match datfile.write_all(&bytes) {
                        Ok(_) => debug!("write to file: {}, bytes: {}", filename, bytes.len()),
                        Err(_) => warn!("problem writing chunk {} to file, ignoring it", pos),
                    };
                } else {
                    warn!("problem creating file for chunk {}, ignoring it", pos)
                }
            }
        }
    }
}

impl<P: Payloads> Client<P> {
    pub(crate) fn maintain_chunks(&self, _mgr: &mut Manager<Self>) {
        if let Some(player_entity) = self.player_entity() {
            // Find the chunk the player is in
            let (player_pos, player_vel);
            {
                let player = player_entity.read();
                player_pos = player.pos().map(|e| e as VoxAbs);
                player_vel = player.vel().map(|e| e as VoxAbs);
            }

            const GENERATION_FACTOR: f32 = 1.0; // generate more than you see
            const GENERATION_SUMMAND: VolOffs = 2; // generate more than you see
            let view_dist = (self.view_distance as f32 * GENERATION_FACTOR) as VolOffs + GENERATION_SUMMAND;
            let view_dist_block = terrain::voloffs_to_voxabs(Vec3::new(view_dist, view_dist, view_dist), CHUNK_SIZE);
            let mut bl = self.chunk_mgr().block_loader_mut();
            bl.clear();
            bl.push(Arc::new(RwLock::new(BlockLoader {
                pos: player_pos,
                size: view_dist_block,
            }))); //player
            bl.push(Arc::new(RwLock::new(BlockLoader {
                pos: player_pos + player_vel * 5,
                size: view_dist_block,
            }))); // player in 5 sec
        }
        //TODO: maybe remove this from CHUNMGR, and just pass it here
        self.chunk_mgr().maintain();
    }
}
