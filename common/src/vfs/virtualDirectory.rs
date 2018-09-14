#![feature(extern_prelude)]
use std::collections::HashMap;
use std::collections::hash_map::Values;
use std::iter::Iterator;
use super::Directory;
use super::File;

//#[derive(Clone)]
pub struct virtualDirectory {
    name: String,
    filemap: HashMap<String, Box<File>>,
    dirmap: HashMap<String, Box<Directory>>,
}

impl Directory for virtualDirectory {
    fn name(&self) -> &str {
        return &self.name;
    }

    fn files(&self) -> Box<Iterator<Item = &Box<File>>> {
        //let mut i: Iterator<Item = Box<File>> = self.filemap.values();
        let mut i = self.filemap.values();
        let mut list = &mut i as &mut Iterator<Item = &Box<File>>;
        return list;
    }

    fn directories(&self) -> &Iterator<Item = &Box<Directory>> {
        let mut i = self.dirmap.values();
        let mut list = &mut i as &mut Iterator<Item = &Box<Directory>>;
        return list;
    }
/*
    fn files<T: Iterator<Item = Box<File>>>(&self) -> T {
        let i: Iterator<Item = Box<File>> = self.filemap.values();
        //return ;
    }

    fn directories(&self) -> Iterator<Item = Box<Directory>> {
        let i: Iterator<Item = Box<Directory>> = self.dirmap.values();
        //return self.dirmap.values() as Iterator<Item = Box<File>>;
    }
*/
    fn getFile(&self) -> Option<Box<File>> {
        None
    }

    fn getDirectory(&self) -> Option<Box<Directory>> {
        None
    }
}
