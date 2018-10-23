// Standard
use std::ops::{Add, Sub, Div, Mul, Neg, Rem};

// Library
use vek::*;
use noise::{NoiseFn, SuperSimplex, HybridMulti, Seedable, MultiFractal};

// Local
use Gen;
use cachegen::CacheGen;

#[derive(Copy, Clone)]
pub struct Sample {
    pub dry: f64,
    pub temp: f64,
    pub chaos: f64,
    pub tree_density: f64,
    pub grad_vari: f64,

    pub hill: f64,
    pub river: f64,
    pub ridge: f64,
    pub cliff_height: f64,
}

pub struct OverworldGen {
    turb_nz: (SuperSimplex, SuperSimplex),
    dry_nz: HybridMulti,
    temp_nz: HybridMulti,
    temp_vari_nz: HybridMulti,
    chaos_nz: SuperSimplex,
    tree_density_nz: HybridMulti,
    grad_vari_nz: SuperSimplex,

    hill_nz: HybridMulti,
    ridge_nz: HybridMulti,
    cliff_height_nz: HybridMulti,
}

impl OverworldGen {
    pub fn new() -> CacheGen<Self> {
        let mut seed = 0;
        let mut new_seed = || { seed += 1; seed };

        CacheGen::new(Self {
            turb_nz: (
                SuperSimplex::new().set_seed(new_seed()),
                SuperSimplex::new().set_seed(new_seed()),
            ),
            dry_nz: HybridMulti::new()
                .set_seed(new_seed())
                .set_octaves(4),
            temp_nz: HybridMulti::new()
                .set_seed(new_seed())
                .set_octaves(3),
            temp_vari_nz: HybridMulti::new()
                .set_seed(new_seed())
                .set_octaves(2),
            tree_density_nz: HybridMulti::new()
                .set_seed(new_seed())
                .set_octaves(3),
            grad_vari_nz: SuperSimplex::new()
                .set_seed(new_seed()),

            chaos_nz: SuperSimplex::new()
                .set_seed(new_seed()),
            hill_nz: HybridMulti::new()
                .set_seed(new_seed())
                .set_octaves(2),
            ridge_nz: HybridMulti::new()
                .set_seed(new_seed())
                .set_octaves(3),
            cliff_height_nz: HybridMulti::new()
                .set_seed(new_seed())
                .set_octaves(3),
        }, 64)
    }

    // 0.0 = wet, 1.0 = dry
    fn get_dry(&self, pos: Vec2<f64>) -> f64 {
        let scale = 3000.0;
        self.dry_nz.get(pos.div(scale).into_array()).mul(1.5).abs().min(1.0)
    }

    // -1.0 = coldest, 0.0 = avg, 1.0 = hottest
    fn get_temp(&self, pos: Vec2<f64>, dry: f64) -> f64 {
        let scale = 4096.0;
        let vari_scale = 32.0;
        // Dryer areas have a less stable temperature
        (
            self.temp_nz.get(pos.div(scale).into_array()) * 0.95 +
            self.temp_vari_nz.get(pos.div(vari_scale).into_array()) * 0.05
        ).mul(0.5 + dry * 0.5)
    }

    // 0.0 = normal/low, 1.0 = high
    fn get_chaos(&self, pos: Vec2<f64>, dry: f64) -> f64 {
        let scale = 900.0;
        self.chaos_nz.get(pos.div(scale).into_array()).mul(dry).powf(1.0).mul(1.3).max(0.0).min(1.0)
    }

    // 0.0 = normal/low, 1.0 = high
    fn get_tree_density(&self, pos: Vec2<f64>, chaos: f64) -> f64 {
        let scale = 1000.0;
        self.tree_density_nz.get(pos.div(scale).into_array()).add(1.0).div(2.0).sub(0.5 * chaos)
    }

    // 0.0 = low, 1.0 = high
    fn get_grad_vari(&self, pos: Vec2<f64>) -> f64 {
        let scale = 32.0;
        self.grad_vari_nz.get(pos.div(scale).into_array()).add(1.0).div(2.0)
    }

    // -amp = lowest, amp = highest
    fn get_hill(&self, pos: Vec2<f64>, dry: f64) -> f64 {
        let scale = 1000.0;
        let amp = 16.0;
        self.hill_nz.get(pos.div(scale).into_array()).mul(amp)
    }

    // 0.0 = normal/flat, max_depth = deepest
    fn get_river(&self, dry: f64, hill: f64) -> f64 {
        let depth = 24.0;
        let max_depth = 8.0 + hill * 0.4;

        if dry < 0.2 {
            dry.mul(20.0).cos().mul(max_depth).max(0.0)
        } else {
            0.0
        }
    }

    // 0.0 = lowest, height = highest
    fn get_ridge(&self, pos: Vec2<f64>, chaos: f64) -> f64 {
        let scale = 1500.0;
        let height = 240.0;
        (1.0 - self.ridge_nz.get(pos.div(scale).into_array()).abs()).powf(0.75).mul(chaos).mul(height)
    }

    // (1.0 - vari) * height = lowest, 1.0 = avg, (1.0 + vari) * height = highest
    fn get_cliff_height(&self, pos: Vec2<f64>) -> f64 {
        let scale = 800.0;
        let vari = 0.6;
        let height = 180.0;

        self.cliff_height_nz.get(pos.div(scale).into_array()).mul(vari).add(1.0).mul(height)
    }
}

impl Gen for OverworldGen {
    type In = Vec2<i64>;
    type Out = Sample;

    fn sample(&self, pos: Vec2<i64>) -> Sample {
        let pos = pos.map(|e| e as f64);

        let turb_scale = 128.0;
        let turb_amp = 64.0;
        let turb_pos = pos + Vec2::new(
            self.turb_nz.0.get(pos.div(turb_scale).into_array()),
            self.turb_nz.1.get(pos.div(turb_scale).into_array()),
        ) * turb_amp;

        let dry = self.get_dry(turb_pos);
        let temp = self.get_temp(pos, dry);
        let chaos = self.get_chaos(pos, dry);
        let tree_density = self.get_tree_density(pos, chaos);
        let grad_vari = self.get_grad_vari(pos);

        let hill = self.get_hill(turb_pos, dry);
        let river = self.get_river(dry, hill);
        let ridge = self.get_ridge(turb_pos, chaos);
        let cliff_height = self.get_cliff_height(pos);

        Sample {
            dry,
            temp,
            chaos,
            tree_density,
            grad_vari,

            hill,
            river,
            ridge,
            cliff_height,
        }
    }
}
