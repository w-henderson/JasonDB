//! Provides the core database API for JasonDB.

use crate::error::JasonError;
use crate::query::Query;
use crate::sources::{FileSource, Source};
use crate::util::indexing;

use humphrey_json::prelude::*;
use humphrey_json::Value;

use std::borrow::Borrow;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::vec::IntoIter;

/// Represents a JasonDB database.
///
/// The type of values in the database is specified by the `T` generic parameter.
/// This must implement Humphrey JSON's [`IntoJson`] and [`FromJson`] traits, which can be done
///   with its [`json_map`] macro.
/// These traits are automatically implemented for basic types like strings and numbers.
///
/// ## Example
/// ```
/// let source = FileSource::new("database.jdb");
/// let mut db: Database<MyType> = Database::new(source)?;
/// ```
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
    /// Creates a new database backed by the given source.
    ///
    /// For file sources, this will index and compact the source, so may take a noticeable amount of time for large databases.
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

    /// Configures the database to use the given secondary index.
    /// This is intended for use in a builder pattern as the example below shows.
    ///
    /// The field can be given as a dot-separated string or using the field macro, and it specifies how to find
    ///   the field to index in the JSON representation of the type.
    ///
    /// ## Example
    /// ```
    /// let mut db = Database::new(source)?
    ///     .index_on(field!(my_field.my_subfield))?
    ///     .index_on("my_field.my_other_subfield")?;
    /// ```
    pub fn index_on(mut self, field: impl AsRef<str>) -> Result<Self, JasonError> {
        let field = field.as_ref().to_string();

        let indexes = self.source.index_on(&field, &self.primary_indexes)?;

        self.secondary_indexes.insert(field, indexes);

        Ok(self)
    }

    /// Gets the value with the given key.
    ///
    /// Returns `Err(JasonError::InvalidKey)` if the index is not found, or another error if the source fails.
    pub fn get(&mut self, key: impl AsRef<str>) -> Result<T, JasonError> {
        let index = *self
            .primary_indexes
            .get(key.as_ref())
            .ok_or(JasonError::InvalidKey)?;

        Ok(self.get_at_index(index)?.1)
    }

    /// Gets the value at the given index.
    /// Returns both the key and the value.
    pub(crate) fn get_at_index(&mut self, index: u64) -> Result<(String, T), JasonError> {
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

    /// Sets the value with the given key to the given value.
    ///
    /// Updates all indexes with the new value.
    pub fn set(&mut self, key: impl AsRef<str>, value: impl Borrow<T>) -> Result<(), JasonError> {
        let json = humphrey_json::to_string(value.borrow());
        let index = self.source.write_entry(key.as_ref(), json.as_bytes())?;
        self.primary_indexes.insert(key.as_ref().to_string(), index);

        for (index_path, indexes) in &mut self.secondary_indexes {
            let indexed_value = indexing::get_value(index_path, &value.borrow().to_json())?;
            indexes
                .entry(indexed_value)
                .or_insert_with(Vec::new)
                .push(index);
        }

        Ok(())
    }

    /// Deletes the value with the given key.
    ///
    /// This appends a null value to the end of the database, and updates all indexes.
    pub fn delete(&mut self, key: impl AsRef<str>) -> Result<(), JasonError> {
        let index = self
            .primary_indexes
            .remove(key.as_ref())
            .ok_or(JasonError::InvalidKey)?;

        let value = self.get_at_index(index)?.1.to_json();

        for (index_path, indexes) in &mut self.secondary_indexes {
            let indexed_value = indexing::get_value(index_path, &value)?;
            indexes
                .get_mut(&indexed_value)
                .ok_or(JasonError::InvalidKey)?
                .retain(|i| *i != index);
        }

        self.source.write_entry(key.as_ref(), "null")?;

        Ok(())
    }

    /// Executes the given query on the database.
    ///
    /// Queries are typically constructed with the `query!` macro.
    pub fn query(&mut self, query: Query) -> Result<Iter<T, S>, JasonError> {
        query.execute(self)
    }

    /// Creates an iterator over the database.
    ///
    /// This only reads from the database when it is used, so is very cheap to create.
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

    /// Migrates the database to a new type according to the function.
    pub fn migrate<U, F>(mut self, f: F) -> Result<Database<U, S>, JasonError>
    where
        U: IntoJson + FromJson,
        F: Fn(T) -> U,
    {
        self.source.migrate(&self.primary_indexes, f)?;

        Database::new(self.source)
    }
}

/// An iterator over the database.
pub struct Iter<'a, T, S>
where
    T: IntoJson + FromJson,
    S: Source,
{
    pub(crate) database: &'a mut Database<T, S>,
    pub(crate) keys: IntoIter<u64>,
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
