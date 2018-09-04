// Library
use coord::prelude::*;

// Project
use region::{
    chunk::{Chunk, ChunkContainer, ChunkFile},
    Container, Key, PersState, VolContainer,
};

// Local
use Client;
use Payloads;
use CHUNK_SIZE;

use std::path::Path;

pub(crate) fn gen_chunk<P: Send + Sync + 'static>(pos: Vec3<i64>, con: &Container<ChunkContainer, P>) {
    let filename = pos.print() + ".dat";
    let filepath = "./saves/".to_owned() + &(filename);
    let path = Path::new(&filepath);
    if path.exists() {
        let mut vol = ChunkFile::new(vec3!(CHUNK_SIZE));
        *vol.file_mut() = filepath;
        con.vols_mut().insert(vol, PersState::File);
    } else {
        let mut vol = Chunk::test(
            vec3!(pos.x * CHUNK_SIZE[0], pos.y * CHUNK_SIZE[1], pos.z * CHUNK_SIZE[2]),
            vec3!(CHUNK_SIZE),
        );
        con.vols_mut().insert(vol, PersState::Raw);
    }
}

impl<P: Payloads> Client<P> {
    pub(crate) fn update_chunks(&self) {
        // Only update chunks if the player exists
        if let Some(player_entity) = self.player_entity() {
            let player_entity = player_entity.write();

            // Find the chunk the player is in
            let player_chunk = player_entity.pos().map(|e| e as i64).div_euc(vec3!(CHUNK_SIZE));

            // Generate chunks around the player first, hacky hack, fix this somehow
            for i in player_chunk.x - 1..player_chunk.x + 2 {
                for j in player_chunk.y - 1..player_chunk.y + 2 {
                    for k in player_chunk.z - 1..player_chunk.z + 2 {
                        let pos = vec3!(i, j, k);
                        if !self.chunk_mgr().contains(pos) {
                            self.chunk_mgr().gen(pos);
                        }
                        if let Some(con) = self.chunk_mgr().persistence().get(&pos) {
                            con.set_access(); // always keep around player
                        }
                    }
                }
            }

            // Generate chunks around the player
            for i in player_chunk.x - self.view_distance..player_chunk.x + self.view_distance + 1 {
                for j in player_chunk.y - self.view_distance..player_chunk.y + self.view_distance + 1 {
                    for k in player_chunk.z - self.view_distance..player_chunk.z + self.view_distance + 1 {
                        let pos = vec3!(i, j, k);
                        if !self.chunk_mgr().contains(pos) {
                            self.chunk_mgr().gen(pos);
                        } else {
                            if self.chunk_mgr().loaded(pos) {
                                self.chunk_mgr().persistence().generate(&pos, PersState::Raw);
                            }
                        }
                    }
                }
            }
        }
    }

    pub(crate) fn lazy_recreate_payload(&self) {
        // Only update payloads if the player exists
        if let Some(player_entity) = self.player_entity() {
            let player_entity = player_entity.write();

            // Find the chunk the player is in
            let player_chunk = player_entity.pos().map(|e| e as i64).div_euc(vec3!(CHUNK_SIZE));

            // Generate payload around the player if it got dropped by persistence
            for i in player_chunk.x - self.view_distance..player_chunk.x + self.view_distance + 1 {
                for j in player_chunk.y - self.view_distance..player_chunk.y + self.view_distance + 1 {
                    for k in player_chunk.z - self.view_distance..player_chunk.z + self.view_distance + 1 {
                        let pos = vec3!(i, j, k);
                        if let Some(con) = self.chunk_mgr().persistence().get(&pos) {
                            if con.payload().is_none() {
                                self.chunk_mgr().gen_payload(pos);
                            }
                        }
                    }
                }
            }
        }
    }
}
