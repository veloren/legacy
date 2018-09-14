use super::Directory;
use super::File;
use std::iter::Iterator;

#[derive(Clone)]
pub struct physicalDirectory {
    name: String,
}
/*
impl Directory for physicalDirectory {
    IterFile = Iterator<Item = Box<File>>;
    IterDirectory = Iterator<Item = Box<Directory>>;

    fn name(&self) -> &str {
        return &self.name;
    }

    fn files(&self) -> IterFile {
        return Iterator::<Item = Box<File>>::new();
    }

    fn directories(&self) -> IterDirectory {
        return Iterator::<Item = Box<File>>::new();
    }

    fn getFile(&self) -> Option<Box<File>> {
        None
    }

    fn getDirectory(&self) -> Option<Box<Directory>> {
        None
    }
}
*/
