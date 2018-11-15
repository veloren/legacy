use std::cmp::{Ord, Ordering, PartialOrd};

/// The indices (order) here should correspond to the ones found in shaders/util/luts.glsl
#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Eq, Hash, EnumMap, PartialOrd, Ord)]
pub enum Material {
    GlossySmooth,
    GlossyRough,
    MatteSmooth,
    MatteRough,
    MetallicSmooth,
    MetallicRough,
    Snow,
    Stone,
    Grass,
    Leaves,
    Earth,
    Log,
    Sand,
    Water,
    Empty,
}

#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum MaterialKind {
    Solid = 0,
    Water = 1,
    Translucent = 2,
    Empty = 3,
}

impl MaterialKind {
    pub fn is_opaque(&self) -> bool {
        match *self {
            MaterialKind::Solid => true,
            _ => false,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct RenderMaterial {
    mat: u8,
    kind: MaterialKind,
}

impl RenderMaterial {
    pub fn new(mat: u8, kind: MaterialKind) -> Self { RenderMaterial { mat, kind } }

    pub fn kind(&self) -> MaterialKind { self.kind }

    pub fn mat(&self) -> u8 { self.mat }

    pub fn is_opaque(&self) -> bool { self.kind.is_opaque() }
}

impl Ord for RenderMaterial {
    fn cmp(&self, other: &RenderMaterial) -> Ordering { self.kind.cmp(&other.kind) }
}

impl PartialOrd for RenderMaterial {
    fn partial_cmp(&self, other: &RenderMaterial) -> Option<Ordering> { Some(self.cmp(other)) }
}
