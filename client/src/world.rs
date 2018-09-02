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

pub(crate) fn gen_chunk<P: Send + Sync + 'static>(pos: Vec2<i64>, con: &Container<ChunkContainer, P>) {
    let filename = pos.print() + ".dat";
    let filepath = "./saves/".to_owned() + &(filename);
    let path = Path::new(&filepath);
    if path.exists() {
        let mut vol = ChunkFile::new(vec3!(CHUNK_SIZE, CHUNK_SIZE, 256));
        *vol.file_mut() = filepath;
        con.vols_mut().insert(vol, PersState::File);
    } else {
        let mut vol = Chunk::test(
            vec3!(pos.x * CHUNK_SIZE, pos.y * CHUNK_SIZE, 0),
            vec3!(CHUNK_SIZE, CHUNK_SIZE, 256),
        );
        con.vols_mut().insert(vol, PersState::Raw);
    }
}

impl<P: Payloads> Client<P> {
    pub(crate) fn update_chunks(&self) {
        // Only update chunks if the player exists
        if let Some(player_entity) = self.player_entity() {
            let player_entity = player_entity.write().unwrap();

            // Find the chunk the player is in
            let player_chunk = player_entity.pos().map(|e| e as i64).div_euc(vec3!([CHUNK_SIZE; 3]));

            // Generate chunks around the player
            for i in player_chunk.x - self.view_distance..player_chunk.x + self.view_distance + 1 {
                for j in player_chunk.y - self.view_distance..player_chunk.y + self.view_distance + 1 {
                    if !self.chunk_mgr().contains(vec2!(i, j)) {
                        self.chunk_mgr().gen(vec2!(i, j));
                    } else {
                        if self.chunk_mgr().loaded(vec2!(i, j)) {
                            self.chunk_mgr().persistence().generate(&vec2!(i, j), PersState::Raw);
                        }
                    }
                }
            }
        }
    }
}
