use vfs::boxed::BoxVPath;

use crate::file::File;

pub struct FileSystemFileEntry {
    path: BoxVPath,
}

impl FileSystemFileEntry {
    pub fn file(&self) -> rquickjs::Result<File> {
        todo!()
    }
}
