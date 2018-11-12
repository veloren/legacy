// Standard
use std::ops::{Add, Div, Mul};

// Library
use noise::{HybridMulti, MultiFractal, NoiseFn, Seedable, SuperSimplex};
use vek::*;

// Local
use new_seed;
use Gen;

pub struct OverworldGen {
    land_nz: HybridMulti,
    dry_nz: HybridMulti,
    temp_nz: HybridMulti,

    hill_nz: HybridMulti,

    temp_vari_nz: SuperSimplex,
    alt_vari_nz: SuperSimplex,
}

#[derive(Copy, Clone)]
pub struct Out {
    pub land: f64,
    pub dry: f64,
    pub temp: f64,

    pub temp_vari: f64,
    pub alt_vari: f64,

    pub z_alt: f64,
    pub z_water: f64,
    pub z_sea: f64,
    pub z_hill: f64,
}

impl OverworldGen {
    pub fn new() -> Self {
        Self {
            // Large-scale
            land_nz: HybridMulti::new().set_seed(new_seed()).set_octaves(8),
            dry_nz: HybridMulti::new().set_seed(new_seed()).set_octaves(7),
            temp_nz: HybridMulti::new().set_seed(new_seed()).set_octaves(8),

            // Small-scale
            hill_nz: HybridMulti::new().set_seed(new_seed()).set_octaves(4),

            temp_vari_nz: SuperSimplex::new().set_seed(new_seed()),
            alt_vari_nz: SuperSimplex::new().set_seed(new_seed()),
        }
    }

    // -1 = deep ocean, 0 = sea level, 1 = mountain
    fn get_land(&self, pos: Vec2<f64>) -> f64 {
        let scale = 3000.0;

        self.land_nz.get(pos.div(scale).into_array())
    }

    fn get_dry(&self, pos: Vec2<f64>) -> f64 {
        let scale = 700.0;
        let vari_scale = scale / 1.0;
        let vari_ampl = scale / 6.0;

        let vari = self.dry_nz.get(pos.div(vari_scale).into_array()) * vari_ampl;

        1.0 - pos.x.add(vari).div(scale).mul(3.14).sin().abs()
    }

    // 0 = cold, 0 = moderate, 1 = hot
    fn get_temp(&self, pos: Vec2<f64>) -> f64 {
        let scale = 10000.0;

        self.temp_nz.get(pos.div(scale).into_array()).add(1.0).div(2.0)
    }

    // 0 = lowest, 1 = highest
    fn get_hill(&self, pos: Vec2<f64>) -> f64 {
        let scale = 1024.0;

        self.hill_nz.get(pos.div(scale).into_array()).add(1.0).div(2.0)
    }

    // 0 = no river, 1 = deep river
    fn get_river(&self, dry: f64) -> f64 {
        let frac = 0.002;
        if dry < frac {
            dry.div(frac).mul(3.14).cos().add(1.0).div(2.0)
        } else {
            0.0
        }
    }
}

impl Gen<()> for OverworldGen {
    type In = Vec2<i64>;
    type Out = Out;

    fn sample(&self, pos: Vec2<i64>, _: &()) -> Out {
        let pos_f64 = pos.map(|e| e as f64) * 1.0;

        let land = self.get_land(pos_f64);
        let dry = self.get_dry(pos_f64);
        let temp = self.get_temp(pos_f64);
        let river = self.get_river(dry);

        let hill = self.get_hill(pos_f64);
        let z_hill = hill * 32.0 * dry.min(land).mul(4.0).min(1.0).max(0.3);

        let z_base = 126.0;
        let z_sea = 118.0;

        let z_land = z_base + land * 32.0;
        let z_height =
            z_land + dry * 192.0 * (1.0 - temp).mul(2.0).min(1.0).max(0.4) * (land * 2.0).min(1.0).max(0.4) + z_hill;
        let z_alt = z_height - river * 8.0;
        let z_water = (z_height - 3.0).max(z_sea);

        Out {
            land,
            dry,
            temp,

            temp_vari: self.temp_vari_nz.get(pos_f64.div(48.0).into_array()),
            alt_vari: self.alt_vari_nz.get(pos_f64.div(32.0).into_array()),

            z_alt,
            z_water,
            z_sea,
            z_hill,
        }
    }
}
