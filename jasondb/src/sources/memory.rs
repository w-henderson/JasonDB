use crate::error::JasonError;
use crate::sources::Source;
use crate::util::{indexing, quiet_assert};

use humphrey_json::prelude::*;
use humphrey_json::Value;

use std::collections::HashMap;

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
    fn read_entry(&mut self, offset: u64) -> Result<(String, Vec<u8>), JasonError> {
        let (k, v_index) = load_value(&self.data, offset)?;
        let (v, _) = load_value(&self.data, v_index as u64)?;

        Ok((
            unsafe { String::from_utf8_unchecked(k.to_vec()) },
            v.to_vec(),
        ))
    }

    fn write_entry(&mut self, k: impl AsRef<str>, v: impl AsRef<[u8]>) -> Result<u64, JasonError> {
        let k = k.as_ref();
        let v = v.as_ref();
        let size = k.len() + v.len() + 16;

        self.data.reserve(size);
        self.data.extend_from_slice(&k.len().to_le_bytes());
        self.data.extend_from_slice(k.as_bytes());
        self.data.extend_from_slice(&v.len().to_le_bytes());
        self.data.extend_from_slice(v);

        Ok((self.data.len() - size) as u64)
    }

    fn load_indexes(&mut self) -> Result<HashMap<String, u64>, JasonError> {
        let mut indexes: HashMap<String, u64> = HashMap::new();
        let mut offset = 0;

        while offset < self.data.len() {
            let (k, v_index) = load_value(&self.data, offset as u64)?;
            let (v, new_offset) = load_value(&self.data, v_index as u64)?;

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

    fn index_on(
        &mut self,
        k: impl AsRef<str>,
        primary_indexes: &HashMap<String, u64>,
    ) -> Result<HashMap<Value, Vec<u64>>, JasonError> {
        let mut indexes: HashMap<Value, Vec<u64>> = HashMap::new();

        for i in primary_indexes.values() {
            let (_, v) = self.read_entry(*i)?;
            let json = unsafe { String::from_utf8_unchecked(v) };
            let value = Value::parse(json).map_err(|_| JasonError::JsonError)?;
            let indexed_value = indexing::get_value(k.as_ref(), &value)?;

            indexes.entry(indexed_value).or_insert(vec![]).push(*i);
        }

        Ok(indexes)
    }

    fn compact(&mut self, indexes: &HashMap<String, u64>) -> Result<(), JasonError> {
        let mut new_data = Vec::new();

        for &start_index in indexes.values() {
            let start_index: usize = start_index.try_into().map_err(|_| JasonError::Index)?;
            let (_, v_index) = load_value(&self.data, start_index as u64)?;
            let (_, end_index) = load_value(&self.data, v_index as u64)?;

            new_data.extend_from_slice(&self.data[start_index..end_index]);
        }

        self.data = new_data;

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
        let mut new_data = InMemory::new();

        for &start_index in indexes.values() {
            let (k, v) = self.read_entry(start_index)?;
            let value_string = unsafe { String::from_utf8_unchecked(v) };

            let old: Old =
                humphrey_json::from_str(&value_string).map_err(|_| JasonError::JsonError)?;
            let new: New = f(old);
            let new_bytes = humphrey_json::to_string(&new).into_bytes();

            new_data.write_entry(k, new_bytes)?;
        }

        *self = new_data;

        Ok(())
    }
}

fn load_value(data: &[u8], offset: u64) -> Result<(&[u8], usize), JasonError> {
    let offset: usize = offset.try_into().map_err(|_| JasonError::Index)?;

    quiet_assert(offset + 8 <= data.len(), JasonError::Index)?;
    let size: usize = u64::from_le_bytes(
        data[offset..offset + 8]
            .try_into()
            .map_err(|_| JasonError::Index)?,
    )
    .try_into()
    .map_err(|_| JasonError::Index)?;
    quiet_assert(offset + 8 + size <= data.len(), JasonError::Index)?;
    let data = &data[offset + 8..offset + 8 + size];

    Ok((data, offset + 8 + size))
}
