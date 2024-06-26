#![allow(dead_code)]

use crate::text::PieceTable;

pub struct FileInfo {
    pub path: std::path::PathBuf,
    pub time: std::time::SystemTime,
}

#[derive(Default)]
pub struct Buffer {
    pub text: PieceTable,
    pub info: Option<FileInfo>,
}

impl Buffer {
    pub fn read(path: std::path::PathBuf) -> std::io::Result<Buffer> {
        let time = std::fs::metadata(&path)?.modified()?;
        let text = PieceTable::from(std::fs::read_to_string(&path)?);
        Ok(Buffer { text, info: Some(FileInfo { path, time }) })
    }
}
