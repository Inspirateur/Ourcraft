use std::ops::{Add, BitXor};
use bevy::prelude::Vec3;
use crate::blocs::{Realm, CHUNK_S1};
use super::{unchunked, chunked, CHUNK_S1I};

#[derive(Clone, Copy, Eq, PartialEq, Default, Debug, Hash)]
pub struct Pos3d<const U: usize> {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub realm: Realm
}

const K: usize = 0x517cc1b727220a95;

impl<const U: usize> Pos3d<U> {
    pub fn dist(&self, other: Pos3d<U>) -> i32 {
        (self.x - other.x).abs()
            .max((self.y - other.y).abs())
            .max((self.z - other.z).abs())
    }

    fn _prng(&self, seed: usize) -> usize {
        (seed)
            .rotate_left(5).bitxor(self.x as usize).wrapping_mul(K)
            .rotate_left(5).bitxor(self.y as usize).wrapping_mul(K)
            .rotate_left(5).bitxor(self.z as usize).wrapping_mul(K)
    }

    pub fn prng(&self, seed: i32) -> usize {
        let n = self._prng(seed as usize);
        self._prng(n)
    }
}

pub type BlocPos = Pos3d<1>;
pub type ChunkPos = Pos3d<CHUNK_S1>;
pub type ChunkedPos = (usize, usize, usize);

impl From<(Vec3, Realm)> for BlocPos {
    fn from((pos, realm): (Vec3, Realm)) -> Self {
        BlocPos {
            x: pos.x.floor() as i32,
            y: pos.y.floor() as i32,
            z: pos.z.floor() as i32,
            realm: realm
        }
    }
}

impl From<BlocPos> for Vec3 {
    fn from(bloc_pos: BlocPos) -> Self {
        Vec3 {
            x: bloc_pos.x as f32, 
            y: bloc_pos.y as f32, 
            z: bloc_pos.z as f32
        }
    }
}

impl Add<Vec3> for BlocPos {
    type Output = BlocPos;

    fn add(self, rhs: Vec3) -> Self::Output {
        BlocPos {
            x: self.x + rhs.x.floor() as i32,
            y: self.y + rhs.y.floor() as i32,
            z: self.z + rhs.z.floor() as i32,
            realm: self.realm
        }
    }
}

impl Add<(i32, i32, i32)> for BlocPos {
    type Output = BlocPos;

    fn add(self, (dx, dy, dz): (i32, i32, i32)) -> Self::Output {
        BlocPos {
            x: self.x + dx,
            y: self.y + dy,
            z: self.z + dz,
            realm: self.realm
        }
    }
}

impl From<(ChunkPos, ChunkedPos)> for BlocPos {
    fn from((chunk_pos, (dx, dy, dz)): (ChunkPos, ChunkedPos)) -> Self {
        BlocPos {
            x: unchunked(chunk_pos.x, dx),
            y: unchunked(chunk_pos.y, dy),
            z: unchunked(chunk_pos.z, dz),
            realm: chunk_pos.realm
        }
    }
}

impl From<BlocPos> for (ChunkPos, ChunkedPos) {
    fn from(bloc_pos: BlocPos) -> Self {
        let (cx, dx) = chunked(bloc_pos.x);
        let (cy, dy) = chunked(bloc_pos.y);
        let (cz, dz) = chunked(bloc_pos.z);
        (ChunkPos {
            x: cx,
            y: cy,
            z: cz,
            realm: bloc_pos.realm
        }, (dx, dy, dz))
    }
}

impl From<BlocPos> for ChunkPos {
    fn from(bloc_pos: BlocPos) -> Self {
        let cx = bloc_pos.x/CHUNK_S1I;
        let cy = bloc_pos.y/CHUNK_S1I;
        let cz = bloc_pos.z/CHUNK_S1I;
        ChunkPos {
            x: cx,
            y: cy,
            z: cz,
            realm: bloc_pos.realm
        }
    }
}