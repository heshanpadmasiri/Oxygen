use std::{
    io::{BufReader, Read},
    path::{Path, PathBuf},
};

// XXX: it is better if the collection module take the view of files as is instead of
// trying to match the gRPC message types
use crate::oxygen::{Collection, File};

pub struct Storage {
    handles: Vec<Handle>,
}

#[derive(Debug)]
pub enum StorageErr {
    NotAFile,
    InvalidPath,
}

/// Use to represent an "item" in storage. All storage APIs will use these as identifiers for directories / files
#[derive(Debug)]
struct Handle {
    handle_type: HandleType,
    path: PathBuf,
    index: usize,
    child_indices: Vec<usize>,
}

#[derive(Debug)]
enum HandleType {
    File,
    Directory,
}
// This is an infallible dereference
trait FromHandle {
    fn from_handle(handle: &Handle, handles: &[Handle]) -> Self;
}
impl FromHandle for File {
    fn from_handle(handle: &Handle, _handles: &[Handle]) -> Self {
        assert!(matches!(handle.handle_type, HandleType::File));
        File {
            name: handle_name(handle),
            id: handle.index as u64,
        }
    }
}

impl FromHandle for Collection {
    fn from_handle(handle: &Handle, handles: &[Handle]) -> Self {
        assert!(matches!(handle.handle_type, HandleType::Directory));
        let mut child_collection_handles = Vec::new();
        let mut child_file_handles = Vec::new();
        for child_index in &handle.child_indices {
            let child = &handles[*child_index];
            match child.handle_type {
                HandleType::Directory => child_collection_handles.push(child),
                HandleType::File => child_file_handles.push(child),
            }
        }
        let child_collections: Vec<Collection> = child_collection_handles
            .iter()
            .map(|handle| Collection::from_handle(handle, handles))
            .collect();
        let files: Vec<File> = child_file_handles
            .iter()
            .map(|handle| File::from_handle(handle, handles))
            .collect();
        Collection {
            name: handle_name(handle),
            id: handle.index as u64,
            child_collections,
            files,
        }
    }
}

fn handle_name(handle: &Handle) -> String {
    handle
        .path
        .file_name()
        .expect("expect handle to have a name")
        .to_str()
        .expect("expect a valid name")
        .to_string()
}

const STORAGE_ROOT: &str = "./test_storage";

/// Starting from the root directory recursively create handles for each file and directory
fn index_storage(root: &Path) -> Result<Vec<Handle>, StorageErr> {
    if !root.exists() {
        return Err(StorageErr::InvalidPath);
    }
    if !root.is_dir() {
        return Err(StorageErr::NotAFile);
    }
    let mut handles = Vec::new();
    index_storage_inner(root, &mut handles);
    Ok(handles)
}

fn index_storage_inner(path: &Path, handles: &mut Vec<Handle>) -> Option<usize> {
    // TODO: properly handle symlink
    if path.is_symlink() {
        return None;
    }
    if path.is_file() {
        if let Some(extension) = path.extension() {
            if extension == "md" {
                let index = handles.len();
                handles.push(Handle {
                    handle_type: HandleType::File,
                    path: path.to_path_buf(),
                    index,
                    child_indices: vec![],
                });
                return Some(index);
            }
        }
        return None;
    }
    let mut child_indices = vec![];
    for entry in path.read_dir().expect("read_dir call failed").flatten() {
        if let Some(index) = index_storage_inner(&entry.path(), handles) {
            child_indices.push(index);
        }
    }
    let index = handles.len();
    handles.push(Handle {
        handle_type: HandleType::Directory,
        path: path.to_path_buf(),
        index,
        child_indices,
    });
    Some(index)
}

impl Storage {
    pub fn new() -> Self
    where
        Self: Sized,
    {
        // FIXME: handle errors when we can have custom root
        Self {
            handles: index_storage(Path::new(STORAGE_ROOT))
                .expect("expect storage initialization to succeed"),
        }
    }

    pub fn get_collection_all(&self) -> Vec<Collection> {
        self.handles
            .iter()
            .filter_map(|handle| match handle.handle_type {
                HandleType::File => None,
                HandleType::Directory => Some(Collection::from_handle(handle, &self.handles)),
            })
            .collect()
    }

    pub fn get_collection(&self, id: u64) -> Result<Collection, ()> {
        let index: usize = id.try_into().unwrap();
        if index < self.handles.len() {
            let handle = &self.handles[index];
            match handle.handle_type {
                HandleType::File => Err(()),
                HandleType::Directory => Ok(Collection::from_handle(handle, &self.handles)),
            }
        } else {
            Err(())
        }
    }

    pub fn get_file(&self, id: u64) -> Result<File, ()> {
        let index: usize = id.try_into().unwrap();
        if index < self.handles.len() {
            let handle = &self.handles[index];
            match handle.handle_type {
                HandleType::File => Ok(File::from_handle(handle, &self.handles)),
                HandleType::Directory => Err(()),
            }
        } else {
            Err(())
        }
    }

    pub fn get_file_content(&self, id: u64) -> Result<Vec<u8>, ()> {
        let index: usize = id.try_into().unwrap();
        if index < self.handles.len() {
            let handle = &self.handles[index];
            match handle.handle_type {
                HandleType::File => Ok(handle_content(handle)),
                HandleType::Directory => Err(()),
            }
        } else {
            Err(())
        }
    }
}

fn handle_content(handle: &Handle) -> Vec<u8> {
    assert!(matches!(handle.handle_type, HandleType::File));
    let file = std::fs::File::open(&handle.path).expect("expect file opening to succeed");
    let mut reader = BufReader::new(file);
    let mut buffer = Vec::new();
    reader
        .read_to_end(&mut buffer)
        .expect("expect file read to succeed");
    buffer
}

#[cfg(test)]
mod tests {

    use crate::{
        oxygen::{Collection, File},
        storage::FromHandle,
    };

    use super::{index_storage, Handle, HandleType, Storage};
    use std::{
        collections::HashSet,
        io::{BufReader, Read},
        path::Path,
    };

    const TEST_STORAGE_ROOT: &str = "./test_storage/";
    #[test]
    fn can_index_test_storage() {
        let handles = index_storage(Path::new(TEST_STORAGE_ROOT))
            .expect("Indexing hardcoded storage must succeed");
        assert!(handles.len() == 10)
    }

    #[test]
    fn can_get_all_collections() {
        let storage = Storage::new();
        let collections = storage.get_collection_all();
        let mut unique_ids = HashSet::new();
        // assert all ids are unique
        assert!(collections
            .iter()
            .map(|collection| { collection.id })
            .all(|id| unique_ids.insert(id)));

        // assert we got all the collections and handle index is same as collection id
        assert!(collection_handles()
            .iter()
            .map(|handle| { handle.index as u64 })
            .all(|index| unique_ids.contains(&index)));
    }

    #[test]
    fn can_get_individual_collection() {
        let storage = Storage::new();
        let handles = handles();
        for handle in collection_handles() {
            let collection = storage
                .get_collection(handle.index as u64)
                .expect("getting collection should not fail");
            assert_eq!(handle.index as u64, collection.id);
            let expected = Collection::from_handle(&handle, &handles);
            assert_eq!(expected, collection);
        }
    }

    #[test]
    fn can_get_individual_files() {
        let storage = Storage::new();
        let handles = handles();
        for handle in file_handles() {
            let file = storage
                .get_file(handle.index as u64)
                .expect("getting file should not fail");
            assert_eq!(handle.index as u64, file.id);
            let expected = File::from_handle(&handle, &handles);
            assert_eq!(expected, file);
        }
    }

    #[test]
    fn can_get_individual_file_content() {
        let storage = Storage::new();
        for handle in file_handles() {
            let actual = storage
                .get_file_content(handle.index as u64)
                .expect("getting file content should not fail");
            let file = std::fs::File::open(&handle.path).expect("expect file opening to succeed");
            let mut reader = BufReader::new(file);
            let mut expected = Vec::new();
            reader
                .read_to_end(&mut expected)
                .expect("expect hardcoded file read to succeed");
            assert_eq!(actual, expected);
        }
    }

    fn collection_handles() -> Vec<Handle> {
        handles()
            .into_iter()
            .filter(|handle| match handle.handle_type {
                HandleType::Directory => true,
                HandleType::File => false,
            })
            .collect()
    }

    fn file_handles() -> Vec<Handle> {
        handles()
            .into_iter()
            .filter(|handle| match handle.handle_type {
                HandleType::Directory => false,
                HandleType::File => true,
            })
            .collect()
    }

    fn handles() -> Vec<Handle> {
        index_storage(Path::new(TEST_STORAGE_ROOT))
            .expect("indexing hardcoded storage must succeed")
    }
}
