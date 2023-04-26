//! Provides backend sources for the database as well as the extensible `Source` trait.

mod file;
mod memory;

pub use file::FileSource;
pub use memory::InMemory;

use crate::error::JasonError;

use humphrey_json::prelude::*;
use humphrey_json::Value;

use std::collections::{BTreeSet, HashMap};

/// Represents a backend source for the database.
///
/// This handles the database's low-level storage API. It is currently implemented for:
///   - [`FileSource`]: A file-based source (default).
///   - [`InMemory`]: A in-memory source with a simple `Vec` as its buffer.
pub trait Source {
    /// Reads an entry from the source at the given offset. Returns its key and value.
    fn read_entry(&mut self, offset: u64) -> Result<(String, Vec<u8>), JasonError>;

    /// Writes an entry to the source with the given key and value. Returns the offset of the new entry.
    fn write_entry(&mut self, k: impl AsRef<str>, v: impl AsRef<[u8]>) -> Result<u64, JasonError>;

    /// Loads indexes from the source. Returns a map of keys to offsets.
    fn load_indexes(&mut self) -> Result<HashMap<String, u64>, JasonError>;

    /// Loads secondary indexes from the source. Returns a map of keys to offsets.
    fn index_on(
        &mut self,
        k: impl AsRef<str>,
        indexes: &HashMap<String, u64>,
    ) -> Result<HashMap<Value, BTreeSet<u64>>, JasonError>;

    /// Compacts the database, removing all deleted entries to save space.
    fn compact(&mut self, indexes: &HashMap<String, u64>) -> Result<(), JasonError>;

    /// Migrates the source from one datatype to another.
    fn migrate<Old, New, F>(
        &mut self,
        indexes: &HashMap<String, u64>,
        f: F,
    ) -> Result<(), JasonError>
    where
        Old: IntoJson + FromJson,
        New: IntoJson + FromJson,
        F: Fn(Old) -> New;
}
