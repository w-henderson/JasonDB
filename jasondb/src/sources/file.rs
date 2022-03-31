use crate::error::JasonError;
use crate::sources::Source;
use crate::util::quiet_assert;

use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

pub struct FileSource {
    pub(crate) file: File,
    pub(crate) path: PathBuf,
    pub(crate) len: u64,
}

impl FileSource {
    pub fn new(path: impl AsRef<Path>) -> Result<Self, JasonError> {
        let file = OpenOptions::new()
            .read(true)
            .create(true)
            .append(true)
            .open(&path)
            .map_err(|_| JasonError::Io)?;

        let meta = file.metadata().map_err(|_| JasonError::Io)?;
        let len = meta.len();

        quiet_assert(meta.is_file(), JasonError::Io)?;

        Ok(Self {
            file,
            path: path.as_ref().to_path_buf(),
            len,
        })
    }

    pub fn create(path: impl AsRef<Path>) -> Result<Self, JasonError> {
        quiet_assert(!path.as_ref().exists(), JasonError::Io)?;
        Self::new(path)
    }

    pub fn open(path: impl AsRef<Path>) -> Result<Self, JasonError> {
        quiet_assert(path.as_ref().exists(), JasonError::Io)?;
        Self::new(path)
    }

    fn load_size(&mut self, offset: u64) -> Result<u64, JasonError> {
        let mut size_buf = [0u8; 8];
        self.file
            .seek(SeekFrom::Start(offset))
            .map_err(|_| JasonError::Index)?;
        self.file
            .read_exact(&mut size_buf)
            .map_err(|_| JasonError::Io)?;

        Ok(u64::from_le_bytes(size_buf))
    }

    fn load_value(&mut self, offset: u64) -> Result<(Vec<u8>, u64), JasonError> {
        let size = self.load_size(offset)?;
        let mut data: Vec<u8> = vec![0; size as usize];
        self.file
            .seek(SeekFrom::Start(offset + 8))
            .map_err(|_| JasonError::Index)?;
        self.file
            .read_exact(&mut data)
            .map_err(|_| JasonError::Io)?;

        Ok((data, offset + 8 + size))
    }
}

impl Source for FileSource {
    fn read_entry(&mut self, offset: u64) -> Result<Vec<u8>, JasonError> {
        let v_index = offset + self.load_size(offset)? + 8;
        let (v, _) = self.load_value(v_index)?;

        Ok(v.to_vec())
    }

    fn write_entry(&mut self, k: impl AsRef<str>, v: impl AsRef<[u8]>) -> Result<u64, JasonError> {
        let k = k.as_ref();
        let v = v.as_ref();
        let size = k.len() + v.len() + 16;

        self.file
            .write_all(&k.len().to_le_bytes())
            .map_err(|_| JasonError::Io)?;
        self.file
            .write_all(k.as_bytes())
            .map_err(|_| JasonError::Io)?;
        self.file
            .write_all(&v.len().to_le_bytes())
            .map_err(|_| JasonError::Io)?;
        self.file.write_all(v).map_err(|_| JasonError::Io)?;

        self.len += size as u64;

        Ok(self.len - size as u64)
    }

    fn load_indexes(&mut self) -> Result<HashMap<String, u64>, JasonError> {
        let mut indexes: HashMap<String, u64> = HashMap::new();
        let mut offset = 0;

        while offset < self.len {
            let (k, v_index) = self.load_value(offset)?;
            let (v, new_offset) = self.load_value(v_index)?;

            let key = unsafe { String::from_utf8_unchecked(k.to_vec()) };

            if v == b"null" {
                indexes.remove(&key);
            } else {
                indexes.insert(key, offset as u64);
            }

            offset = new_offset;
        }

        Ok(indexes)
    }

    fn compact(&mut self, indexes: &HashMap<String, u64>) -> Result<(), JasonError> {
        let temp_path = self.path.with_file_name("__jdb_temp");
        if temp_path.exists() {
            fs::remove_file(&temp_path).map_err(|_| JasonError::Io)?;
        }

        let mut new_file = OpenOptions::new()
            .create_new(true)
            .append(true)
            .open(&temp_path)
            .map_err(|_| JasonError::Io)?;
        let mut new_len: u64 = 0;

        for &start_index in indexes.values() {
            let v_index = start_index + self.load_size(start_index)? + 8;
            let end_index = v_index + self.load_size(v_index)? + 8;

            let mut buf: Vec<u8> = vec![0; (end_index - start_index) as usize];
            self.file
                .seek(SeekFrom::Start(start_index))
                .map_err(|_| JasonError::Index)?;
            self.file.read_exact(&mut buf).map_err(|_| JasonError::Io)?;

            new_file.write_all(&buf).map_err(|_| JasonError::Io)?;
            new_len += buf.len() as u64;
        }

        drop(new_file);

        fs::rename(&self.path, self.path.with_file_name("__jdb_old"))
            .map_err(|_| JasonError::Io)?;
        fs::rename(&temp_path, &self.path).map_err(|_| JasonError::Io)?;

        let new_file = OpenOptions::new()
            .read(true)
            .append(true)
            .open(&self.path)
            .map_err(|_| JasonError::Io)?;

        let _old_file = std::mem::replace(&mut self.file, new_file);
        self.len = new_len;

        fs::remove_file(self.path.with_file_name("__jdb_old")).map_err(|_| JasonError::Io)?;

        Ok(())
    }
}
