mod memory;

pub use memory::InMemory;

use std::collections::HashMap;
use std::error::Error;
use std::mem::size_of;

pub const LEN_SIZE: usize = size_of::<usize>();

pub trait Source {
    fn read_entry(&self, offset: usize) -> Result<Vec<u8>, Box<dyn Error>>;
    fn write_entry(
        &mut self,
        k: impl AsRef<str>,
        v: impl AsRef<[u8]>,
    ) -> Result<usize, Box<dyn Error>>;
    fn load_indexes(&self) -> Result<HashMap<String, usize>, Box<dyn Error>>;
    fn compact(&mut self, indexes: &HashMap<String, usize>) -> Result<(), Box<dyn Error>>;
}
