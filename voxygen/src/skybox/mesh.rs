// Library
use vek::*;

// Local
use voxel::{RenderVolume, RenderVoxel};

gfx_defines! {
    vertex Vertex {
        pos: [f32; 3] = "vert_pos",
    }
}

impl Vertex {
    pub fn new(pos: [f32; 3]) -> Vertex { Vertex { pos } }
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

    pub fn flat(p0: [f32; 3], p1: [f32; 3], p2: [f32; 3], p3: [f32; 3]) -> Quad {
        Quad {
            verts: [
                Vertex { pos: p0 },
                Vertex { pos: p1 },
                Vertex { pos: p2 },
                Vertex { pos: p3 },
            ],
        }
    }
}

pub struct Mesh {
    verts: Vec<Vertex>,
}

impl Mesh {
    pub fn new() -> Mesh { Mesh { verts: Vec::new() } }

    pub fn new_skybox() -> Mesh {
        let mut mesh = Mesh::new();
        mesh.add_quads(&[
            Quad::flat(
                // -x
                [-1.0, -1.0, -1.0],
                [-1.0, 1.0, -1.0],
                [-1.0, 1.0, 1.0],
                [-1.0, -1.0, 1.0],
            ),
            Quad::flat(
                // +x
                [1.0, -1.0, 1.0],
                [1.0, 1.0, 1.0],
                [1.0, 1.0, -1.0],
                [1.0, -1.0, -1.0],
            ),
            Quad::flat(
                // -y
                [1.0, -1.0, -1.0],
                [-1.0, -1.0, -1.0],
                [-1.0, -1.0, 1.0],
                [1.0, -1.0, 1.0],
            ),
            Quad::flat(
                // +y
                [1.0, 1.0, 1.0],
                [-1.0, 1.0, 1.0],
                [-1.0, 1.0, -1.0],
                [1.0, 1.0, -1.0],
            ),
            Quad::flat(
                // -z
                [-1.0, -1.0, -1.0],
                [1.0, -1.0, -1.0],
                [1.0, 1.0, -1.0],
                [-1.0, 1.0, -1.0],
            ),
            Quad::flat(
                // +z
                [-1.0, 1.0, 1.0],
                [1.0, 1.0, 1.0],
                [1.0, -1.0, 1.0],
                [-1.0, -1.0, 1.0],
            ),
        ]);
        mesh
    }

    #[allow(dead_code)]
    pub fn vert_count(&self) -> u32 { self.verts.len() as u32 }

    #[allow(dead_code)]
    pub fn vertices(&self) -> &Vec<Vertex> { &self.verts }

    pub fn add(&mut self, verts: &[Vertex]) { self.verts.extend_from_slice(verts); }

    pub fn add_quads(&mut self, quads: &[Quad]) {
        for q in quads {
            self.add(&[q.verts[0], q.verts[1], q.verts[2], q.verts[2], q.verts[3], q.verts[0]]);
        }
    }
}
