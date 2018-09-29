// Library
use vek::*;

// Project
use common::manager::Manager;
use region::Chunk;

// Local
use Client;
use Payloads;
use CHUNK_SIZE;

pub(crate) fn gen_chunk(pos: Vec2<i64>) -> Chunk {
    Chunk::test(
        Vec3::new(pos.x * CHUNK_SIZE, pos.y * CHUNK_SIZE, 0),
        Vec3::new(CHUNK_SIZE, CHUNK_SIZE, 256),
    )
}

impl<P: Payloads> Client<P> {
    pub(crate) fn update_chunks(&self, mgr: &mut Manager<Self>) {
        // Only update chunks if the player exists
        if let Some(player_entity) = self.player_entity() {
            let player_entity = player_entity.write();

            // Find the chunk the player is in
            let player_chunk = player_entity.pos().map(|e| e as i64).map(|e| e.div_euc(CHUNK_SIZE));

            // Generate chunks around the player
            for i in player_chunk.x - self.view_distance..player_chunk.x + self.view_distance + 1 {
                for j in player_chunk.y - self.view_distance..player_chunk.y + self.view_distance + 1 {
                    if !self.chunk_mgr().contains(Vec2::new(i, j)) {
                        self.chunk_mgr().gen(Vec2::new(i, j));
                    }
                }
            }

            // Remove chunks that are too far from the player
            // TODO: Could be more efficient (maybe? careful: deadlocks)
            let pers = self.chunk_mgr().persistence();
            let chunk_pos = pers.data().keys().map(|p| *p).collect::<Vec<_>>();
            for pos in chunk_pos {
                if (pos - Vec2::new(player_chunk.x, player_chunk.y))
                    .map(|e| e as f32)
                    .magnitude()
                    > self.view_distance as f32 * 2.0
                {
                    Manager::add_worker(mgr, move |client, _, _| {
                        client.chunk_mgr().remove(pos);
                    });
                }
            }
        }
    }
}
