// Standard
use std::{collections::HashMap, ops::Range};

// Library
use serde_derive::{Deserialize, Serialize};
use specs::{
    saveload::{Marker, MarkerAllocator},
    world::EntitiesRes,
    Component, DenseVecStorage, Entity, Join, ReadStorage,
};

// The marker components and marker allocator here are used
// to map entities with a unique ID (UidMarker) that is consistent
// between client and server. This is done because both client and
// server may have their own entities that screw up allocation of
// `Entity` ids.

// SyncMarker

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct UidMarker {
    id: u64,
    seq: u64,
}

impl Component for UidMarker {
    type Storage = DenseVecStorage<Self>;
}

impl Marker for UidMarker {
    type Identifier = u64;
    type Allocator = UidNode;

    fn id(&self) -> u64 { self.id }

    fn update(&mut self, update: Self) {
        assert_eq!(self.id, update.id);
        self.seq = update.seq;
    }
}

// SyncNode

pub struct UidNode {
    pub(crate) range: Range<u64>,
    pub(crate) mapping: HashMap<u64, Entity>,
}

impl MarkerAllocator<UidMarker> for UidNode {
    fn allocate(&mut self, entity: Entity, id: Option<u64>) -> UidMarker {
        let id = id.unwrap_or_else(|| self.range.next().expect("Id range must be virtually endless"));
        self.mapping.insert(id, entity);
        UidMarker { id, seq: 0 }
    }

    fn retrieve_entity_internal(&self, id: u64) -> Option<Entity> { self.mapping.get(&id).cloned() }

    fn maintain(&mut self, entities: &EntitiesRes, storage: &ReadStorage<UidMarker>) {
        self.mapping = (&*entities, storage).join().map(|(e, m)| (m.id(), e)).collect();
    }
}
