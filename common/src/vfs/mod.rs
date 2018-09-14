mod physicalFile;
mod physicalDirectory;
mod virtualDirectory;
use std::collections::HashMap;
use std::iter::Iterator;
// Reexports
pub use self::{
//    physicalDirectory,
};

/*
### Virtual File System ###
This is a virtual filesystem which will allow us to represent the local filesystem as well as network data as files to the game
*/

pub trait File {
    fn name(&self) -> &str;
    fn load(&self) -> Vec<u8>;
    fn save(&self, data: Vec<u8>);
    fn append(&self, data: Vec<u8>);
}

pub trait Directory {
    fn name(&self) -> &str;
    fn files(&self) -> Box<Iterator<Item = &Box<File>>>;
    fn directories(&self) -> &Iterator<Item = &Box<Directory>>;
    fn getFile(&self) -> Option<Box<File>>;
    fn getDirectory(&self) -> Option<Box<Directory>>;
}
