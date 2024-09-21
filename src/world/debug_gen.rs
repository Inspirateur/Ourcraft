use crate::world::{
    MAX_GEN_HEIGHT, CHUNK_S1,
    unchunked, VoxelWorld, ColPos
};
use crate::{Block, block::Soils};
use itertools::iproduct;
use nd_interval::NdInterval;
use std::{collections::HashMap, path::Path};

pub struct DebugGen {
    seed: u32,
    config: HashMap<String, f32>,
    soils: Soils,
}

impl Clone for DebugGen {
    fn clone(&self) -> Self {
        DebugGen::new(self.seed, self.config.clone())
    }
}

fn values(x: i32, z: i32) -> (f32, f32, f32) {
    let y = ((x as f32/50.).sin()*0.5+0.5+(z as f32/50.).cos()*0.5+0.5)/2.;
    (0.5, 0.5, 0.5)
}

impl DebugGen {
    pub fn new(seed: u32, config: std::collections::HashMap<String, f32>) -> Self
    where
        Self: Sized + Clone,
    {
        DebugGen {
            seed,
            config,
            soils: Soils::from_csv(Path::new("assets/data/soils_condition.csv")).unwrap(),
        }
    }

    pub fn gen(&self, world: &VoxelWorld, col: ColPos) {
        for (dx, dz) in iproduct!(0..CHUNK_S1, 0..CHUNK_S1) {
            let (x, z) = (unchunked(col.x, dx), unchunked(col.z, dz));
            let (y, t, h) = values(x, z);
            let y = (y * MAX_GEN_HEIGHT as f32) as i32;
            assert!(y >= 0);
            let block = *self.soils.closest([t as f32, h as f32]).unwrap_or((&Block::Dirt, 0.)).0;
            world.set_yrange(col, (dx, dz), y, 3, block);
        }
        // this is a bit too slow so we don't bother with it for now
        // col.fill_up(Block::Stone);
    }
}
