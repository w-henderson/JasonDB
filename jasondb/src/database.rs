use crate::error::JasonError;
use crate::sources::{FileSource, Source};

use humphrey_json::prelude::*;
use humphrey_json::Value;

use std::borrow::Borrow;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::vec::IntoIter;

pub struct Database<T, S = FileSource>
where
    T: IntoJson + FromJson,
    S: Source,
{
    pub(crate) primary_indexes: HashMap<String, u64>,
    pub(crate) secondary_indexes: HashMap<String, HashMap<Value, Vec<u64>>>,
    pub(crate) source: S,
    marker: PhantomData<T>,
}

impl<T, S> Database<T, S>
where
    T: IntoJson + FromJson,
    S: Source,
{
    pub fn new(mut source: S) -> Result<Self, JasonError> {
        let indexes = source.load_indexes()?;
        source.compact(&indexes)?;
        let indexes = source.load_indexes()?;

        Ok(Self {
            primary_indexes: indexes,
            secondary_indexes: HashMap::new(),
            source,
            marker: PhantomData,
        })
    }

    pub fn get(&mut self, key: impl AsRef<str>) -> Result<T, JasonError> {
        let index = *self
            .primary_indexes
            .get(key.as_ref())
            .ok_or(JasonError::InvalidKey)?;

        Ok(self.get_at_index(index)?.1)
    }

    fn get_at_index(&mut self, index: u64) -> Result<(String, T), JasonError> {
        let (k, v) = self.source.read_entry(index)?;
        let json = unsafe { String::from_utf8_unchecked(v) };

        if json == "null" {
            Err(JasonError::InvalidKey)
        } else {
            Ok((
                k,
                humphrey_json::from_str(json).map_err(|_| JasonError::JsonError)?,
            ))
        }
    }

    pub fn set(&mut self, key: impl AsRef<str>, value: impl Borrow<T>) -> Result<(), JasonError> {
        let json = humphrey_json::to_string(value.borrow());
        let index = self.source.write_entry(key.as_ref(), json.as_bytes())?;
        self.primary_indexes.insert(key.as_ref().to_string(), index);

        Ok(())
    }

    pub fn delete(&mut self, key: impl AsRef<str>) -> Result<(), JasonError> {
        self.primary_indexes
            .remove(key.as_ref())
            .ok_or(JasonError::InvalidKey)?;
        self.source.write_entry(key.as_ref(), "null")?;

        Ok(())
    }

    pub fn iter(&mut self) -> Iter<T, S> {
        let keys = self
            .primary_indexes
            .values()
            .cloned()
            .collect::<Vec<_>>()
            .into_iter();

        Iter {
            database: self,
            keys,
        }
    }
}

pub struct Iter<'a, T, S>
where
    T: IntoJson + FromJson,
    S: Source,
{
    database: &'a mut Database<T, S>,
    keys: IntoIter<u64>,
}

impl<'a, T, S> Iterator for Iter<'a, T, S>
where
    T: IntoJson + FromJson,
    S: Source,
{
    type Item = Result<(String, T), JasonError>;

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.keys.next()?;
        let value = self.database.get_at_index(index);

        Some(value)
    }
}
