// Standard
use std::time::Duration;

// Library
use vek::*;

// Project
use common::{
    audio::{Position, Stream},
    terrain::{chunk::Block, VoxAbs},
    util::manager::Manager,
};

// Local
use crate::{Client, Payloads};

impl<P: Payloads> Client<P> {
    pub(crate) fn maintain_music(&self, _mgr: &mut Manager<Self>) {
        //ambient sound
        let start_tick = *self.next_ambient.read();
        let clock_tick_time = *self.clock_tick_time.read();
        if start_tick < clock_tick_time {
            //check player pos
            let mut player_pos = None;
            let mut player_vel = None;
            if let Some(player_entity) = self.player_entity() {
                let player = player_entity.read();
                player_pos = Some(player.pos().map(|e| e as VoxAbs));
                player_vel = Some(player.vel().map(|e| e as VoxAbs));
            }
            let mut is_water_nearby = false;
            if let Some(player_pos) = player_pos {
                let low = player_pos - Vec3::new(20, 20, 20);
                let high = player_pos + Vec3::new(20, 20, 20);
                if let Ok(volsample) = self.chunk_mgr.try_get_sample(low, high) {
                    for (_, b) in volsample.iter() {
                        if b == Block::WATER {
                            is_water_nearby = true;
                            break;
                        }
                    }
                }
            };

            let duration;
            let buffer;
            if is_water_nearby {
                duration = Duration::from_secs(90);
                buffer = 1;
            } else {
                duration = Duration::from_secs(160);
                buffer = 0;
            }
            self.audio_mgr.gen_stream(Stream {
                buffer,
                start_tick: clock_tick_time,
                duration,
                volume: 0.5,
                repeat: None,
                positional: None,
                fading: None,
            });
            *self.next_ambient.write() = clock_tick_time + duration;
        }

        // entity sounds
        let player_id = self.player.read().entity_uid;
        let start_tick = *self.next_steps.read();
        if start_tick < clock_tick_time {
            let duration = Duration::from_millis(600);
            let entities = self.entities.read();
            for (id, e) in entities.iter() {
                let pos;
                let vel;
                {
                    let lock = e.read();
                    pos = *lock.pos();
                    vel = *lock.vel();
                }
                let mut positional = Some(Position {
                    relative: false,
                    pos,
                    vel,
                });
                if let Some(player_id) = player_id {
                    if player_id == *id {
                        positional = Some(Position {
                            relative: true,
                            pos: Vec3::new(0.0, 0.0, 0.0),
                            vel,
                        });
                    }
                }
                if vel.magnitude_squared() > 0.17 && vel.z.abs() < 3.0 {
                    //some movement on ground
                    self.audio_mgr.gen_stream(Stream {
                        buffer: 2,
                        start_tick: clock_tick_time,
                        duration,
                        volume: 0.25,
                        repeat: None,
                        positional,
                        fading: None,
                    });
                }
            }
            *self.next_steps.write() = clock_tick_time + duration / 2;
        }

        self.audio_mgr.maintain(clock_tick_time);
    }
}
