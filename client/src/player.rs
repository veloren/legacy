// Project
use common::Uid;

pub struct Player {
    pub alias: String,
    pub entity_uid: Option<Uid>,
}

impl Player {
    pub fn new(alias: String) -> Player {
        Player {
            alias,
            entity_uid: None,
        }
    }

    pub fn control_entity(&mut self, entity: Uid) { self.entity_uid = Some(entity); }

    pub fn alias(&self) -> &String { &self.alias }
    pub fn alias_mut(&mut self) -> &mut String { &mut self.alias }

    pub fn entity_uid(&self) -> Option<Uid> { self.entity_uid }
}
