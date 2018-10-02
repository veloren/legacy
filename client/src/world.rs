// Standard
use std::path::Path;

// Library
use vek::*;

// Project
use common::{
    terrain::{
        chunk::{Chunk, ChunkContainer, ChunkConverter, ChunkFile},
        Container, Key, PersState, VolContainer, VolConverter,
    },
    util::manager::Manager,
};

// Local
use Client;
use Payloads;
use CHUNK_SIZE;

pub(crate) fn gen_chunk<P: Send + Sync + 'static>(pos: Vec3<i64>, con: &Container<ChunkContainer, P>) {
    let filename = pos.print() + ".dat";
    let filepath = "./saves/".to_owned() + &(filename);
    let path = Path::new(&filepath);
    if path.exists() {
        let mut vol = ChunkFile::new(Vec3::from_slice(&CHUNK_SIZE));
        *vol.file_mut() = filepath;
        con.vols_mut().insert(vol, PersState::File);
    } else {
        let mut vol = Chunk::test(
            Vec3::new(pos.x * CHUNK_SIZE[0], pos.y * CHUNK_SIZE[1], pos.z * CHUNK_SIZE[2]),
            Vec3::from_slice(&CHUNK_SIZE),
        );
        con.vols_mut().insert(vol, PersState::Raw);
    }
}

impl<P: Payloads> Client<P> {
    pub(crate) fn load_unload_chunks(&self, mgr: &mut Manager<Self>) {
        // Only update chunks if the player exists
        if let Some(player_entity) = self.player_entity() {
            // Find the chunk the player is in
            let player_chunk = player_entity.read().pos().map(|e| e as i64);

            let player_chunk = Vec3::new(
                player_chunk.x.div_euc(CHUNK_SIZE[0]),
                player_chunk.y.div_euc(CHUNK_SIZE[1]),
                player_chunk.z.div_euc(CHUNK_SIZE[2]),
            );

            // Collect chunks around the player
            const GENERATION_FACTOR: f32 = 1.4;
            let mut chunks = vec![];
            let view_dist = (self.view_distance as f32 * GENERATION_FACTOR) as i64;
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
            const MAX_CHUNKS_IN_QUEUE: u64 = 12; // to not overkill the vol_mgr
            for (_diff, pos) in chunks.iter() {
                if !self.chunk_mgr().contains(*pos) {
                    // generate up to MAX_CHUNKS_IN_QUEUE chunks around the player
                    if self.chunk_mgr().pending_cnt() < MAX_CHUNKS_IN_QUEUE as usize {
                        self.chunk_mgr().gen(*pos);
                    }
                } else {
                    // check if payloads does not exist, and then generate it because its dropped below
                    if self.chunk_mgr().loaded(*pos) {
                        self.chunk_mgr().persistence().generate(&pos, PersState::Raw);
                        if let Some(con) = self.chunk_mgr().persistence().get(&pos) {
                            if con.payload().is_none() {
                                self.chunk_mgr().gen_payload(*pos);
                            }
                        }
                    }
                }
            }

            const DIFF_TILL_UNLOAD: i64 = 5;
            //unload chunks that have a distance of 5 or greater that the last rendered chunk, so that we dont unload to fast, e.g. if we go back a chunk
            let unload_chunk_diff = chunks.last().unwrap().0 + DIFF_TILL_UNLOAD;

            //drop old chunks
            {
                let chunks = self.chunk_mgr().persistence().hot();
                for (pos, container) in chunks.iter() {
                    let diff = (player_chunk - *pos).map(|e| e.abs()).sum();
                    if diff > unload_chunk_diff {
                        let mut lock = container.vols_mut();
                        ChunkConverter::convert(pos, &mut lock, PersState::File);
                        lock.remove(PersState::Raw);
                        lock.remove(PersState::Rle);
                        *container.payload_mut() = None;
                    }
                }
            }
        }
    }
}
