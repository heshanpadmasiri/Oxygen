use std::path::{Path, PathBuf};

// XXX: it is better if the collection module take the view of files as is instead of
// trying to match the gRPC message types
use crate::oxygen::{Collection, File, FileContent};

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
            handles: index_storage(&Path::new(STORAGE_ROOT))
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

    pub fn get_file_content(&self, id: u64) -> Result<FileContent, ()> {
        match self.get_file(id) {
            Ok(file) => {
                let body = format!("# {} content", file.name).as_bytes().to_vec();
                Ok(FileContent { body })
            }
            Err(_) => Err(()),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::index_storage;
    use std::path::Path;

    const TEST_STORAGE_ROOT: &str = "./test_storage/";
    #[test]
    fn can_index_test_storage() {
        let handles = index_storage(Path::new(TEST_STORAGE_ROOT))
            .expect("Indexing hardcoded storage must succeed");
        assert!(handles.len() == 10)
    }
}
