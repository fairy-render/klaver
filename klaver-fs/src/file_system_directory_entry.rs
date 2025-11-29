use std::path::PathBuf;

use rquickjs::Result;

use crate::file_system_file_entry::FileSystemFileEntry;

pub struct FileSystemDirectoryEntry {
    path: PathBuf,
}

impl FileSystemDirectoryEntry {
    pub fn get_file(&self) -> Result<FileSystemFileEntry> {
        todo!()
    }

    pub fn get_dir(&self) -> Result<FileSystemDirectoryEntry> {
        todo!()
    }
}
