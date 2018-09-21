// Standard
use std::{
    rc::Rc,
    hash::{Hash, Hasher},
    collections::hash_map::{HashMap, DefaultHasher},
};

// Library
use vek::*;
use lyon::tessellation::geometry_builder::VertexBuffers;

// Local
use shader::Shader;
use super::render::{FillPso, FillVertex};

// What is this?
// -------------
// This is `ResCache`, a cache for UI resources. When we want a resource - let's say a rectangle
// mesh - we look here first, and only generate a new one if it doesn't already exist. Cool, right?
// Most of this is pretty bog-standard utility code that implements the same logic: hash the
// attributes of whatever is being looked up, then returns it if it exists, or creates it if it
// doesn't. Don't bother yourself with caring about this code. It's pretty boring run-of-the-mill
// stuff that would just 'exist' if we didn't decide to write this engine ourselves.

pub struct ResCache {
    // PSOs
    fill_pso: Option<Rc<FillPso>>,
    // Meshes
    rect_meshes: HashMap<u64, Rc<VertexBuffers<FillVertex, u16>>>,
}

impl ResCache {
    pub fn new() -> ResCache {
        ResCache {
            fill_pso: None,
            rect_meshes: HashMap::new(),
        }
    }

    // Example
    //pub(crate) fn get_or_create_x<F: FnOnce() -> X>(&mut self, hash: u64, f: F) -> X

    pub(crate) fn get_or_create_fill_pso<F: FnOnce() -> FillPso>(&mut self, f: F) -> Rc<FillPso> {
        if let None = self.fill_pso {
            self.fill_pso = Some(Rc::new(f()));
        }
        self.fill_pso.as_ref().map(|f| f.clone()).expect("This panic shouldn't be possible.")
    }

    pub(crate) fn get_or_create_rect_mesh<F: FnOnce() -> VertexBuffers<FillVertex, u16>>(&mut self, pos: Vec2<f32>, sz: Vec2<f32>, col: Rgba<f32>, f: F) -> Rc<VertexBuffers<FillVertex, u16>> {
        // Eurgh. Awful hashing logic here.
        let mut hasher = DefaultHasher::new();
        (
            pos.map(|e| e.to_bits()),
            sz.map(|e| e.to_bits()),
            col.map(|e| e.to_bits()),
        ).hash(&mut hasher);
        let hash = hasher.finish();

        if let None = self.rect_meshes.get(&hash) {
            self.rect_meshes.insert(hash, Rc::new(f()));
        }
        self.rect_meshes.get(&hash).map(|r| r.clone()).expect("This panic shouldn't be possible.")
    }
}
