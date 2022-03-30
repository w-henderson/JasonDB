use crate::sources::{Source, LEN_SIZE};
use crate::util::quiet_assert;

use std::collections::HashMap;
use std::error::Error;

#[derive(Default)]
pub struct InMemory {
    pub(crate) data: Vec<u8>,
}

impl InMemory {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Source for InMemory {
    fn read_entry(&self, offset: usize) -> Result<Vec<u8>, Box<dyn Error>> {
        let (_, v_index) = load_value(&self.data, offset)?;
        let (v, _) = load_value(&self.data, v_index)?;

        Ok(v.to_vec())
    }

    fn write_entry(
        &mut self,
        k: impl AsRef<str>,
        v: impl AsRef<[u8]>,
    ) -> Result<usize, Box<dyn Error>> {
        let k = k.as_ref();
        let v = v.as_ref();
        let size = k.len() + v.len() + LEN_SIZE * 2;

        self.data.reserve(size);
        self.data.extend_from_slice(&k.len().to_le_bytes());
        self.data.extend_from_slice(k.as_bytes());
        self.data.extend_from_slice(&v.len().to_le_bytes());
        self.data.extend_from_slice(v);

        Ok(self.data.len() - size)
    }

    fn load_indexes(&self) -> Result<HashMap<String, usize>, Box<dyn Error>> {
        let mut indexes: HashMap<String, usize> = HashMap::new();
        let mut offset = 0;

        while offset < self.data.len() {
            let (k, v_index) = load_value(&self.data, offset)?;
            let (_, new_offset) = load_value(&self.data, v_index)?;

            indexes.insert(unsafe { String::from_utf8_unchecked(k.to_vec()) }, offset);
            offset = new_offset;
        }

        Ok(indexes)
    }

    fn compact(&mut self, indexes: &HashMap<String, usize>) -> Result<(), Box<dyn Error>> {
        let mut new_data = Vec::new();

        for &start_index in indexes.values() {
            let (_, v_index) = load_value(&self.data, start_index)?;
            let (_, end_index) = load_value(&self.data, v_index)?;

            new_data.extend_from_slice(&self.data[start_index..end_index]);
        }

        self.data = new_data;

        Ok(())
    }
}

fn load_value(data: &[u8], offset: usize) -> Result<(&[u8], usize), Box<dyn Error>> {
    quiet_assert(offset + LEN_SIZE <= data.len())?;
    let size = usize::from_le_bytes(data[offset..offset + LEN_SIZE].try_into()?);
    quiet_assert(offset + LEN_SIZE + size <= data.len())?;
    let data = &data[offset + LEN_SIZE..offset + LEN_SIZE + size];

    Ok((data, offset + LEN_SIZE + size))
}
