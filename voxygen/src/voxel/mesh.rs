// Library
use coord::prelude::*;
use fnv::FnvBuildHasher;
use gfx;
use gfx_device_gl;
use indexmap::IndexMap;

type FnvIndexMap<K, V> = IndexMap<K, V, FnvBuildHasher>;

// Project
use region::Voxel;

// Local
use voxel::{Material, MaterialKind, RenderVolume, RenderVoxel};

#[derive(Debug, Clone, Copy)]
pub enum NormalDirection {
    PlusX,
    MinusX,
    PlusY,
    MinusY,
    PlusZ,
    MinusZ,
}

impl From<Vec3<i64>> for NormalDirection {
    fn from(vec: Vec3<i64>) -> Self {
        let elements = vec.elements();
        let idx = elements.iter().position(|e| *e != 0).unwrap();
        let e = elements[idx];
        match (idx, e) {
            (0, e) if e > 0 => NormalDirection::PlusX,
            (0, e) if e < 0 => NormalDirection::MinusX,
            (1, e) if e > 0 => NormalDirection::PlusY,
            (1, e) if e < 0 => NormalDirection::MinusY,
            (2, e) if e > 0 => NormalDirection::PlusZ,
            (2, e) if e < 0 => NormalDirection::MinusZ,
            _ => unreachable!(),
        }
    }
}

impl From<NormalDirection> for u8 {
    fn from(norm: NormalDirection) -> Self {
        match norm {
            NormalDirection::PlusX => 0,
            NormalDirection::MinusX => 1,
            NormalDirection::PlusY => 2,
            NormalDirection::MinusY => 3,
            NormalDirection::PlusZ => 4,
            NormalDirection::MinusZ => 5,
        }
    }
}

gfx_defines! {
    vertex Vertex {
        pos: [f32; 3] = "vert_pos",
        norm_ao_col_mat: u32 = "vert_norm_ao_col_mat",
    }
}

pub(super) type VertexBuffer = gfx::handle::Buffer<gfx_device_gl::Resources, Vertex>;

impl Vertex {
    pub fn new(pos: [f32; 3], norm: NormalDirection, ao: u8, col: u8, mat: Material) -> Vertex {
        let mut norm_ao_col_mat: u32 = u8::from(norm) as _;
        norm_ao_col_mat = norm_ao_col_mat | (ao as u32) << 8;
        norm_ao_col_mat = norm_ao_col_mat | (col as u32) << 16;
        norm_ao_col_mat = norm_ao_col_mat | ((mat as u8) as u32) << 24;
        Vertex { pos, norm_ao_col_mat }
    }

    pub fn scale(&self, scale: Vec3<f32>) -> Vertex {
        Vertex {
            pos: [self.pos[0] * scale.x, self.pos[1] * scale.y, self.pos[2] * scale.z],
            norm_ao_col_mat: self.norm_ao_col_mat,
        }
    }
}

#[derive(Copy, Clone)]
pub struct Poly {
    verts: [Vertex; 3],
}

impl Poly {
    #[allow(dead_code)]
    pub fn new(v0: Vertex, v1: Vertex, v2: Vertex) -> Poly { Poly { verts: [v0, v1, v2] } }
}

#[derive(Copy, Clone)]
pub struct Quad {
    verts: [Vertex; 4],
}

impl Quad {
    pub fn new(v0: Vertex, v1: Vertex, v2: Vertex, v3: Vertex) -> Quad {
        Quad {
            verts: [v0, v1, v2, v3],
        }
    }

    pub fn scale(&self, scale: Vec3<f32>) -> Quad {
        Quad {
            verts: [
                self.verts[0].scale(scale),
                self.verts[1].scale(scale),
                self.verts[2].scale(scale),
                self.verts[3].scale(scale),
            ],
        }
    }

    #[allow(dead_code)]
    pub fn flat_with_color(
        p0: [f32; 3],
        p1: [f32; 3],
        p2: [f32; 3],
        p3: [f32; 3],
        norm: NormalDirection,
        ao: u8,
        col: u8,
        mat: Material,
    ) -> Quad {
        Quad {
            verts: [
                Vertex::new(p0, norm, ao, col, mat),
                Vertex::new(p1, norm, ao, col, mat),
                Vertex::new(p2, norm, ao, col, mat),
                Vertex::new(p3, norm, ao, col, mat),
            ],
        }
    }

    pub fn with_offset(&self, off: [f32; 3]) -> Quad {
        let mut nquad = *self;
        nquad.verts[0].pos = [
            nquad.verts[0].pos[0] + off[0],
            nquad.verts[0].pos[1] + off[1],
            nquad.verts[0].pos[2] + off[2],
        ];
        nquad.verts[1].pos = [
            nquad.verts[1].pos[0] + off[0],
            nquad.verts[1].pos[1] + off[1],
            nquad.verts[1].pos[2] + off[2],
        ];
        nquad.verts[2].pos = [
            nquad.verts[2].pos[0] + off[0],
            nquad.verts[2].pos[1] + off[1],
            nquad.verts[2].pos[2] + off[2],
        ];
        nquad.verts[3].pos = [
            nquad.verts[3].pos[0] + off[0],
            nquad.verts[3].pos[1] + off[1],
            nquad.verts[3].pos[2] + off[2],
        ];
        nquad
    }
}

trait GetAO {
    fn get_ao_at(&self, pos: Vec3<i64>, dir: Vec3<i64>) -> u8;
    fn get_ao_quad(
        &self,
        pos: Vec3<i64>,
        x_unit: Vec3<i64>,
        y_unit: Vec3<i64>,
        z_unit: Vec3<i64>,
        col: u8,
        mat: Material,
    ) -> Quad;
}
impl<V: RenderVolume> GetAO for V
where
    V::VoxelType: RenderVoxel,
{
    fn get_ao_at(&self, pos: Vec3<i64>, dir: Vec3<i64>) -> u8 {
        let vecs = if dir.x == 0 {
            if dir.y == 0 {
                [vec3!(0, 0, 0), vec3!(-1, 0, 0), vec3!(0, -1, 0), vec3!(-1, -1, 0)]
            } else {
                [vec3!(0, 0, 0), vec3!(-1, 0, 0), vec3!(0, 0, -1), vec3!(-1, 0, -1)]
            }
        } else {
            [vec3!(0, 0, 0), vec3!(0, -1, 0), vec3!(0, 0, -1), vec3!(0, -1, -1)]
        };
        vecs.iter().fold(0, |acc, v| {
            acc + if self.at(pos + *v).unwrap_or(V::VoxelType::empty()).is_opaque() {
                0
            } else {
                1
            }
        })
    }

    fn get_ao_quad(
        &self,
        pos: Vec3<i64>,
        x_unit: Vec3<i64>,
        y_unit: Vec3<i64>,
        z_unit: Vec3<i64>,
        col: u8,
        mat: Material,
    ) -> Quad {
        let units = [vec3!(0, 0, 0), x_unit, x_unit + y_unit, y_unit];

        let ao = [
            self.get_ao_at(pos + units[0], z_unit),
            self.get_ao_at(pos + units[1], z_unit),
            self.get_ao_at(pos + units[2], z_unit),
            self.get_ao_at(pos + units[3], z_unit),
        ];

        if ao[0] + ao[2] > ao[1] + ao[3] {
            Quad::new(
                Vertex::new(units[0].map(|e| e as f32).elements(), z_unit.into(), ao[0], col, mat),
                Vertex::new(units[1].map(|e| e as f32).elements(), z_unit.into(), ao[1], col, mat),
                Vertex::new(units[2].map(|e| e as f32).elements(), z_unit.into(), ao[2], col, mat),
                Vertex::new(units[3].map(|e| e as f32).elements(), z_unit.into(), ao[3], col, mat),
            )
        } else {
            Quad::new(
                Vertex::new(units[1].map(|e| e as f32).elements(), z_unit.into(), ao[1], col, mat),
                Vertex::new(units[2].map(|e| e as f32).elements(), z_unit.into(), ao[2], col, mat),
                Vertex::new(units[3].map(|e| e as f32).elements(), z_unit.into(), ao[3], col, mat),
                Vertex::new(units[0].map(|e| e as f32).elements(), z_unit.into(), ao[0], col, mat),
            )
        }
    }
}

pub struct Mesh {
    verts: Vec<Vertex>,
}

impl Mesh {
    pub fn new() -> Mesh { Mesh { verts: Vec::new() } }

    pub fn from<V: RenderVolume>(vol: &V) -> FnvIndexMap<MaterialKind, Mesh>
    where
        V::VoxelType: RenderVoxel,
    {
        Mesh::from_with_offset(vol, vec3!(0.0, 0.0, 0.0))
    }

    pub fn from_with_offset<V: RenderVolume>(vol: &V, offs: Vec3<f32>) -> FnvIndexMap<MaterialKind, Mesh>
    where
        V::VoxelType: RenderVoxel,
    {
        let mut map = FnvIndexMap::with_capacity_and_hasher(4, Default::default());
        let scale = vol.scale();
        let scale = Vec3::new(scale.x as f32, scale.y as f32, scale.z as f32);

        for x in 0..vol.size().x {
            for y in 0..vol.size().y {
                for z in 0..vol.size().z {
                    let vox = vol.at(vec3!(x, y, z)).expect("Attempted to mesh voxel outside volume");
                    let offset = Vec3::new(
                        (x as f32 + offs.x) * scale.x,
                        (y as f32 + offs.y) * scale.y,
                        (z as f32 + offs.z) * scale.z,
                    );

                    let col = vox.get_color();
                    let render_mat = vox.get_mat();
                    let mat = render_mat.mat();

                    let mesh = map.entry(render_mat.kind()).or_insert(Mesh::new());

                    if vox.is_occupied() {
                        let opaque = vox.is_opaque();
                        // +x
                        if vol
                            .at(vec3!(x + 1, y, z))
                            .unwrap_or(V::VoxelType::empty())
                            .should_add(opaque)
                        {
                            mesh.add_quads(&[vol
                                .get_ao_quad(
                                    vec3!(x + 1, y + 0, z + 0),
                                    vec3!(0, 1, 0),
                                    vec3!(0, 0, 1),
                                    vec3!(1, 0, 0),
                                    col,
                                    mat,
                                ).scale(vec3!(scale.x, scale.y, scale.z))
                                .with_offset([offset.x + scale.x, offset.y, offset.z])]);
                        }
                        // -x
                        if vol
                            .at(vec3!(x - 1, y, z))
                            .unwrap_or(V::VoxelType::empty())
                            .should_add(opaque)
                        {
                            mesh.add_quads(&[vol
                                .get_ao_quad(
                                    vec3!(x - 1, y + 0, z + 0),
                                    vec3!(0, 0, 1),
                                    vec3!(0, 1, 0),
                                    vec3!(-1, 0, 0),
                                    col,
                                    mat,
                                ).scale(vec3!(scale.x, scale.y, scale.z))
                                .with_offset([offset.x, offset.y, offset.z])]);
                        }
                        // +y
                        if vol
                            .at(vec3!(x, y + 1, z))
                            .unwrap_or(V::VoxelType::empty())
                            .should_add(opaque)
                        {
                            mesh.add_quads(&[vol
                                .get_ao_quad(
                                    vec3!(x + 0, y + 1, z + 0),
                                    vec3!(0, 0, 1),
                                    vec3!(1, 0, 0),
                                    vec3!(0, 1, 0),
                                    col,
                                    mat,
                                ).scale(vec3!(scale.x, scale.y, scale.z))
                                .with_offset([offset.x, offset.y + scale.y, offset.z])]);
                        }
                        // -y
                        if vol
                            .at(vec3!(x, y - 1, z))
                            .unwrap_or(V::VoxelType::empty())
                            .should_add(opaque)
                        {
                            mesh.add_quads(&[vol
                                .get_ao_quad(
                                    vec3!(x + 0, y - 1, z + 0),
                                    vec3!(1, 0, 0),
                                    vec3!(0, 0, 1),
                                    vec3!(0, -1, 0),
                                    col,
                                    mat,
                                ).scale(vec3!(scale.x, scale.y, scale.z))
                                .with_offset([offset.x, offset.y, offset.z])]);
                        }
                        // +z
                        if vol
                            .at(vec3!(x, y, z + 1))
                            .unwrap_or(V::VoxelType::empty())
                            .should_add(opaque)
                        {
                            mesh.add_quads(&[vol
                                .get_ao_quad(
                                    vec3!(x + 0, y + 0, z + 1),
                                    vec3!(1, 0, 0),
                                    vec3!(0, 1, 0),
                                    vec3!(0, 0, 1),
                                    col,
                                    mat,
                                ).scale(vec3!(scale.x, scale.y, scale.z))
                                .with_offset([offset.x, offset.y, offset.z + scale.z])]);
                        }
                        // -z
                        if vol
                            .at(vec3!(x, y, z - 1))
                            .unwrap_or(V::VoxelType::empty())
                            .should_add(opaque)
                        {
                            mesh.add_quads(&[vol
                                .get_ao_quad(
                                    vec3!(x + 0, y + 0, z - 1),
                                    vec3!(0, 1, 0),
                                    vec3!(1, 0, 0),
                                    vec3!(0, 0, -1),
                                    col,
                                    mat,
                                ).scale(vec3!(scale.x, scale.y, scale.z))
                                .with_offset([offset.x, offset.y, offset.z])]);
                        }
                    }
                }
            }
        }

        map
    }

    #[allow(dead_code)]
    pub fn vert_count(&self) -> u32 { self.verts.len() as u32 }

    #[allow(dead_code)]
    pub fn vertices(&self) -> &Vec<Vertex> { &self.verts }

    pub fn add(&mut self, verts: &[Vertex]) { self.verts.extend_from_slice(verts); }

    #[allow(dead_code)]
    pub fn add_polys(&mut self, polys: &[Poly]) {
        for p in polys {
            self.verts.extend_from_slice(&p.verts);
        }
    }

    pub fn add_quads(&mut self, quads: &[Quad]) {
        for q in quads {
            self.add(&[q.verts[0], q.verts[1], q.verts[2], q.verts[2], q.verts[3], q.verts[0]]);
        }
    }
}
