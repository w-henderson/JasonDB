use crate::error::JasonError;
use crate::sources::{InMemory, Source};
use crate::util::{indexing, quiet_assert};

use humphrey_json::prelude::*;
use humphrey_json::Value;

use std::collections::{BTreeSet, HashMap};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

/// Represents a file-based database source.
///
/// ## Example
/// ```
/// let source = FileSource::new("database.jdb");
/// let mut db: Database<String> = Database::new(source)?;
/// ```
pub struct FileSource {
    pub(crate) file: File,
    pub(crate) path: PathBuf,
    pub(crate) len: u64,
}

impl FileSource {
    /// Opens the file-based database source from the given path, or creates an empty one if it doesn't exist.
    ///
    /// To create an empty database and throw an error if it already exists, use `FileSource::create`.
    /// To open an existing database and throw an error if it doesn't exist, use `FileSource::open`.
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

    /// Creates a new empty file-based database source at the given path.
    ///
    /// If the file already exists, an error will be thrown.
    pub fn create(path: impl AsRef<Path>) -> Result<Self, JasonError> {
        quiet_assert(!path.as_ref().exists(), JasonError::Io)?;
        Self::new(path)
    }

    /// Opens an existing file-based database source at the given path.
    ///
    /// If the file doesn't exist, an error will be thrown.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, JasonError> {
        quiet_assert(path.as_ref().exists(), JasonError::Io)?;
        Self::new(path)
    }

    /// Converts the file source into an in-memory source by copying the contents of the file into memory.
    ///
    /// **Warning:** changes made to the new in-memory source will not be reflected in the original file source. If you're looking
    ///   to have an in-memory database which remains synchronized with the original file source, add a replica to replicate writes
    ///   back to the file, as follows:
    ///
    /// ```rs
    /// let mut db = Database::open("database.jdb")?          // Open the file-based database
    ///     .into_memory()?                                   // Copy into memory
    ///     .with_replica(Database::open("database.jdb")?);   // Replicate subsequent writes back to the file
    /// ```
    pub fn into_memory(mut self) -> Result<InMemory, JasonError> {
        let mut buf: Vec<u8> = Vec::with_capacity(self.len as usize);

        self.file
            .seek(SeekFrom::Start(0))
            .map_err(|_| JasonError::Io)?;
        self.file
            .read_to_end(&mut buf)
            .map_err(|_| JasonError::Io)?;

        Ok(InMemory { data: buf })
    }

    /// Loads the size of a database entry from the given offset.
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

    /// Loads an arbitrary value from the data at the given offset.
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
    fn read_entry(&mut self, offset: u64) -> Result<(String, Vec<u8>), JasonError> {
        let (k, v_index) = self.load_value(offset)?;
        let (v, _) = self.load_value(v_index)?;

        Ok((unsafe { String::from_utf8_unchecked(k) }, v))
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

            let key = unsafe { String::from_utf8_unchecked(k) };

            if v == b"null" {
                indexes.remove(&key);
            } else {
                indexes.insert(key, offset as u64);
            }

            offset = new_offset;
        }

        Ok(indexes)
    }

    fn index_on(
        &mut self,
        k: impl AsRef<str>,
        primary_indexes: &HashMap<String, u64>,
    ) -> Result<HashMap<Value, BTreeSet<u64>>, JasonError> {
        let mut indexes: HashMap<Value, BTreeSet<u64>> = HashMap::new();

        for i in primary_indexes.values() {
            let (_, v) = self.read_entry(*i)?;
            let json = unsafe { String::from_utf8_unchecked(v) };
            let value = Value::parse(json).map_err(|_| JasonError::JsonError)?;
            let indexed_value = indexing::get_value(k.as_ref(), &value);

            indexes
                .entry(indexed_value)
                .or_insert_with(BTreeSet::new)
                .insert(*i);
        }

        Ok(indexes)
    }

    fn compact(&mut self, indexes: &HashMap<String, u64>) -> Result<(), JasonError> {
        let temp_path = self.path.with_extension("jdbtmp");
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

        fs::rename(&self.path, self.path.with_extension("jdbold")).map_err(|_| JasonError::Io)?;
        fs::rename(&temp_path, &self.path).map_err(|_| JasonError::Io)?;

        let new_file = OpenOptions::new()
            .read(true)
            .append(true)
            .open(&self.path)
            .map_err(|_| JasonError::Io)?;

        let _old_file = std::mem::replace(&mut self.file, new_file);
        self.len = new_len;

        fs::remove_file(self.path.with_extension("jdbold")).map_err(|_| JasonError::Io)?;

        Ok(())
    }

    fn migrate<Old, New, F>(
        &mut self,
        indexes: &HashMap<String, u64>,
        f: F,
    ) -> Result<(), JasonError>
    where
        Old: IntoJson + FromJson,
        New: IntoJson + FromJson,
        F: Fn(Old) -> New,
    {
        let temp_path = self.path.with_extension("jdbtmp");
        if temp_path.exists() {
            fs::remove_file(&temp_path).map_err(|_| JasonError::Io)?;
        }

        let mut new_file = FileSource::create(&temp_path)?;

        for &start_index in indexes.values() {
            let (k, v) = self.read_entry(start_index)?;
            let value_string = unsafe { String::from_utf8_unchecked(v) };

            let old: Old =
                humphrey_json::from_str(&value_string).map_err(|_| JasonError::JsonError)?;
            let new: New = f(old);
            let new_bytes = humphrey_json::to_string(&new).into_bytes();

            new_file.write_entry(k, new_bytes)?;
        }

        let new_len = new_file.len;

        drop(new_file);

        fs::rename(&self.path, self.path.with_extension("jdbold")).map_err(|_| JasonError::Io)?;
        fs::rename(&temp_path, &self.path).map_err(|_| JasonError::Io)?;

        let new_file = OpenOptions::new()
            .read(true)
            .append(true)
            .open(&self.path)
            .map_err(|_| JasonError::Io)?;

        let _old_file = std::mem::replace(&mut self.file, new_file);
        self.len = new_len;

        fs::remove_file(self.path.with_extension("jdbold")).map_err(|_| JasonError::Io)?;

        Ok(())
    }
}
