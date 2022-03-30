mod file;
mod memory;

pub use file::FileSource;
pub use memory::InMemory;

use std::collections::HashMap;
use std::error::Error;

pub trait Source {
    fn read_entry(&mut self, offset: u64) -> Result<Vec<u8>, Box<dyn Error>>;
    fn write_entry(
        &mut self,
        k: impl AsRef<str>,
        v: impl AsRef<[u8]>,
    ) -> Result<u64, Box<dyn Error>>;
    fn load_indexes(&mut self) -> Result<HashMap<String, u64>, Box<dyn Error>>;
    fn compact(&mut self, indexes: &HashMap<String, u64>) -> Result<(), Box<dyn Error>>;
}
