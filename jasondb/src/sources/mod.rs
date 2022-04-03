mod file;
mod memory;

pub use file::FileSource;
pub use memory::InMemory;

use crate::error::JasonError;

use humphrey_json::prelude::*;
use humphrey_json::Value;

use std::collections::HashMap;

pub trait Source {
    fn read_entry(&mut self, offset: u64) -> Result<(String, Vec<u8>), JasonError>;
    fn write_entry(&mut self, k: impl AsRef<str>, v: impl AsRef<[u8]>) -> Result<u64, JasonError>;
    fn load_indexes(&mut self) -> Result<HashMap<String, u64>, JasonError>;
    fn index_on(
        &mut self,
        k: impl AsRef<str>,
        indexes: &HashMap<String, u64>,
    ) -> Result<HashMap<Value, Vec<u64>>, JasonError>;

    fn compact(&mut self, indexes: &HashMap<String, u64>) -> Result<(), JasonError>;
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
