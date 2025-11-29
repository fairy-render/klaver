use std::path::PathBuf;

use rquickjs::Result;
use vfs::boxed::BoxVPath;

use crate::file_system_file_entry::FileSystemFileEntry;

pub struct FileSystemDirectoryEntry {
    path: BoxVPath,
}

impl FileSystemDirectoryEntry {
    pub async fn get_file(&self) -> Result<FileSystemFileEntry> {
        todo!()
    }

    pub async fn get_dir(&self) -> Result<FileSystemDirectoryEntry> {
        todo!()
    }
}
