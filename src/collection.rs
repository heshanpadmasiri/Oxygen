// XXX: it is better if the collection module take the view of files as is instead of
// trying to match the gRPC message types
use crate::oxygen::{Collection, File, FileContent};

pub trait Storage {
    // TODO: this needs to be a singleton
    fn new() -> Self
    where
        Self: Sized;
    fn get_collection_all(&self) -> Vec<Collection>;
    // TODO: this needs to return proper errors
    fn get_collection(&self, id: u64) -> Result<Collection, ()>;
    // TODO: this needs to return proper errors
    fn get_file(&self, id: u64) -> Result<File, ()>;
    fn get_file_content(&self, id: u64) -> Result<FileContent, ()>;
}

pub struct HardCodedStorage {
    hard_coded_collections: Vec<Collection>,
    hard_coded_files: Vec<File>,
}

/// Hardcoded file structure
/// collection 4
/// -- collection 3
/// -- -- collection 2
/// -- -- -- f 3.md
/// -- -- -- f_4.md
/// -- -- collection 1
/// -- -- -- collection_0
/// -- -- f 2.md
/// -- f_1.md
impl Storage for HardCodedStorage {
    fn new() -> Self
    where
        Self: Sized,
    {
        let collection_0 = Collection {
            name: "collection_1".to_string(),
            id: 0,
            child_collections: vec![],
            files: vec![],
        };
        let f_2 = File {
            name: "f 2.md".to_string(),
            id: 0,
        };
        let collection_1 = Collection {
            name: "collection 1".to_string(),
            id: 1,
            child_collections: vec![collection_0.clone()],
            files: vec![f_2.clone()],
        };
        let f_3 = File {
            name: "f 3.md".to_string(),
            id: 1,
        };
        let f_4 = File {
            name: "f_4.md".to_string(),
            id: 2,
        };
        let collection_2 = Collection {
            name: "collection 2".to_string(),
            id: 2,
            child_collections: vec![],
            files: vec![f_3.clone(), f_4.clone()],
        };
        let collection_3 = Collection {
            name: "collection 3".to_string(),
            id: 3,
            child_collections: vec![collection_2.clone(), collection_1.clone()],
            files: vec![f_2.clone()],
        };
        let f_1 = File {
            name: "f_1.md".to_string(),
            id: 3,
        };
        let collection_4 = Collection {
            name: "collection 4".to_string(),
            id: 4,
            child_collections: vec![collection_3.clone()],
            files: vec![f_1.clone()],
        };
        let hard_coded_collections = vec![
            collection_0,
            collection_1,
            collection_2,
            collection_3,
            collection_4,
        ];
        let hard_coded_files = vec![f_1, f_2, f_3, f_4];
        Self {
            hard_coded_collections,
            hard_coded_files,
        }
    }

    fn get_collection_all(&self) -> Vec<Collection> {
        self.hard_coded_collections.clone()
    }

    fn get_collection(&self, id: u64) -> Result<Collection, ()> {
        let index: usize = id.try_into().unwrap();
        if index < self.hard_coded_collections.len() {
            Ok(self.hard_coded_collections[index].clone())
        } else {
            Err(())
        }
    }

    fn get_file(&self, id: u64) -> Result<File, ()> {
        let index: usize = id.try_into().unwrap();
        if index < self.hard_coded_files.len() {
            Ok(self.hard_coded_files[index].clone())
        } else {
            Err(())
        }
    }

    fn get_file_content(&self, id: u64) -> Result<FileContent, ()> {
        match self.get_file(id) {
            Ok(file) => {
                let body = format!("# {} content", file.name).as_bytes().to_vec();
                Ok(FileContent { body })
            }
            Err(_) => Err(()),
        }
    }
}
