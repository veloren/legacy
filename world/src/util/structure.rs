// Standard
use std::ops::{Add, Sub, Div, Mul, Neg, Rem};

// Library
use vek::*;

// Local
use Gen;

pub fn dist_by_euc(p: Vec2<i64>) -> i64 {
    (p * p).sum()
}

pub fn dist_by_axis(p: Vec2<i64>) -> i64 {
    p.map(|e| e.abs()).reduce_max()
}

pub struct StructureGen {
    freq: u64,
    warp: u64,
    seed: u32,
    dist_func: fn(p: Vec2<i64>) -> i64,
}

impl StructureGen {
    pub fn new(freq: u64, warp: u64, seed: u32, dist_func: fn(p: Vec2<i64>) -> i64) -> Self

    {
        Self {
            freq,
            warp,
            seed,
            dist_func,
        }
    }

    pub fn throw_dice(&self, pos: Vec2<i64>, seed: u32) -> u64 {
        // TODO: Make this actually good
        let next = 327387278321 ^ (self.seed + seed) as u64 * 1103515245 + 15341;
        let next = 327387278322 ^ (pos.x + 3232782181) as u64 * 1103515245 + 12343;
        let next = 327387278321 ^ (next + (pos.y + 23728323237) as u64) * 1103515245 + 12541;
        next
    }
}

impl<'a, T: Clone, S, F> Gen<(&'a S, F)> for StructureGen
    where F: Fn(&Self, Vec2<i64>, &S) -> T + Send + Sync + 'static
{
    type In = Vec2<i64>;
    type Out = T;

    fn sample(&self, pos: Vec2<i64>, (supplement, f): &(&S, F)) -> T {
        impl StructureGen {
            fn cell_pos(&self, cell_coord: Vec2<i64>) -> Vec2<i64> {
                cell_coord * self.freq as i64 + self.freq as i64 / 2 + if self.warp > 0 {
                    Vec2::new(
                        self.throw_dice(cell_coord, 1337),
                        self.throw_dice(cell_coord, 1338),
                    ).map(|e| (e.mod_euc(self.warp)) as i64) - self.warp as i64 / 2
                } else {
                    Vec2::zero()
                }
            }
        }

        let pos2di = Vec2::<i64>::from(pos);

        let cell_coord = pos2di.map(|e| e.div_euc(self.freq as i64));
        let cell_offs = pos2di.map(|e| e.mod_euc(self.freq as i64)) - self.freq as i64 / 2;

        // TODO: Manually unroll this? Or not? Check to see if the compiler does automatically.
        let mut min = (cell_coord, std::i64::MAX);
        for x in -1..2 {
            for y in -1..2 {
                let cell_pos = self.cell_pos(cell_coord + Vec2::new(x, y));
                let dist = (self.dist_func)(cell_pos.sub(pos2di));
                if dist < min.1 {
                    min = (cell_pos, dist);
                }
            }
        }

        f(self, min.0, supplement)
    }
}
