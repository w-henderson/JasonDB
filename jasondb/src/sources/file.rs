use crate::sources::Source;

use std::collections::HashMap;
use std::error::Error;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

pub struct FileSource {
    pub(crate) file: File,
    pub(crate) path: PathBuf,
    pub(crate) len: u64,
}

impl FileSource {
    pub fn new(path: impl AsRef<Path>) -> Result<Self, Box<dyn Error>> {
        let file = OpenOptions::new()
            .read(true)
            .create(true)
            .append(true)
            .open(&path)?;

        let len = file.metadata()?.len();

        Ok(Self {
            file,
            path: path.as_ref().to_path_buf(),
            len,
        })
    }

    fn load_size(&mut self, offset: u64) -> Result<u64, Box<dyn Error>> {
        let mut size_buf = [0u8; 8];
        self.file.seek(SeekFrom::Start(offset))?;
        self.file.read_exact(&mut size_buf)?;

        Ok(u64::from_le_bytes(size_buf))
    }

    fn load_value(&mut self, offset: u64) -> Result<(Vec<u8>, u64), Box<dyn Error>> {
        let size = self.load_size(offset)?;
        let mut data: Vec<u8> = vec![0; size as usize];
        self.file.seek(SeekFrom::Start(offset + 8))?;
        self.file.read_exact(&mut data)?;

        Ok((data, offset + 8 + size))
    }
}

impl Source for FileSource {
    fn read_entry(&mut self, offset: u64) -> Result<Vec<u8>, Box<dyn Error>> {
        let v_index = offset + self.load_size(offset)? + 8;
        let (v, _) = self.load_value(v_index)?;

        Ok(v.to_vec())
    }

    fn write_entry(
        &mut self,
        k: impl AsRef<str>,
        v: impl AsRef<[u8]>,
    ) -> Result<u64, Box<dyn Error>> {
        let k = k.as_ref();
        let v = v.as_ref();
        let size = k.len() + v.len() + 16;

        self.file.write_all(&k.len().to_le_bytes())?;
        self.file.write_all(k.as_bytes())?;
        self.file.write_all(&v.len().to_le_bytes())?;
        self.file.write_all(v)?;

        self.len += size as u64;

        Ok(self.len - size as u64)
    }

    fn load_indexes(&mut self) -> Result<HashMap<String, u64>, Box<dyn Error>> {
        let mut indexes: HashMap<String, u64> = HashMap::new();
        let mut offset = 0;

        while offset < self.len {
            let (k, v_index) = self.load_value(offset)?;
            let new_offset = v_index + self.load_size(v_index)? + 8;

            indexes.insert(
                unsafe { String::from_utf8_unchecked(k.to_vec()) },
                offset as u64,
            );
            offset = new_offset;
        }

        Ok(indexes)
    }

    fn compact(&mut self, indexes: &HashMap<String, u64>) -> Result<(), Box<dyn Error>> {
        let temp_path = self.path.with_file_name("__jdb_temp");
        if temp_path.exists() {
            fs::remove_file(&temp_path)?;
        }

        let mut new_file = OpenOptions::new()
            .create_new(true)
            .append(true)
            .open(&temp_path)?;
        let mut new_len: u64 = 0;

        for &start_index in indexes.values() {
            let v_index = start_index + self.load_size(start_index)? + 8;
            let end_index = v_index + self.load_size(v_index)? + 8;

            let mut buf: Vec<u8> = vec![0; (end_index - start_index) as usize];
            self.file.seek(SeekFrom::Start(start_index))?;
            self.file.read_exact(&mut buf)?;

            new_file.write_all(&buf)?;
            new_len += buf.len() as u64;
        }

        drop(new_file);

        fs::rename(&self.path, self.path.with_file_name("__jdb_old"))?;
        fs::rename(&temp_path, &self.path)?;

        let new_file = OpenOptions::new()
            .read(true)
            .append(true)
            .open(&self.path)?;

        let _old_file = std::mem::replace(&mut self.file, new_file);
        self.len = new_len;

        fs::remove_file(self.path.with_file_name("__jdb_old"))?;

        Ok(())
    }
}
