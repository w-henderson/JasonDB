use crate::error::JasonError;
use crate::sources::{FileSource, Source};

use humphrey_json::prelude::*;

use std::borrow::Borrow;
use std::collections::HashMap;
use std::marker::PhantomData;

pub struct Database<T, S = FileSource>
where
    T: IntoJson + FromJson,
    S: Source,
{
    pub(crate) indexes: HashMap<String, u64>,
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
            indexes,
            source,
            marker: PhantomData,
        })
    }

    pub fn get(&mut self, key: impl AsRef<str>) -> Result<T, JasonError> {
        let index = self
            .indexes
            .get(key.as_ref())
            .ok_or(JasonError::InvalidKey)?;
        let json = unsafe { String::from_utf8_unchecked(self.source.read_entry(*index)?) };
        let value: T = humphrey_json::from_str(json).map_err(|_| JasonError::JsonError)?;

        Ok(value)
    }

    pub fn set(&mut self, key: impl AsRef<str>, value: impl Borrow<T>) -> Result<(), JasonError> {
        let json = humphrey_json::to_string(value.borrow());
        let index = self.source.write_entry(key.as_ref(), json.as_bytes())?;
        self.indexes.insert(key.as_ref().to_string(), index);

        Ok(())
    }
}
