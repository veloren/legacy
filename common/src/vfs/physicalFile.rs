use super::File;

#[derive(Clone)]
pub struct physicalFile {
    name: String,
}

impl File for physicalFile {
    fn name(&self) -> &str {
        return &self.name;
    }

    fn load(&self) -> Vec<u8> {
        return Vec::new();
    }

    fn save(&self, data: Vec<u8>) {

    }

    fn append(&self, data: Vec<u8>) {

    }
}
