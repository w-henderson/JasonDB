#![allow(dead_code)]
use std::{error::Error, fmt::Display};

#[cfg(feature = "validation")]
use serde_json::{from_str, Value};

use crate::id::generate_id;

/// Struct representing the database as a whole.
/// Contains the collections as well as its name.
///
/// ## Example:
/// ```rs
/// let database = Database::new("myDatabase");
/// database.createCollection("users");
/// database.collection("users").list() // returns an empty vec
/// ```
#[derive(Debug, PartialEq, Eq)]
pub struct Database {
    name: String,
    collections: Vec<Collection>,
    writes: u64,
}

/// Struct representing a collection in the database.
/// Similarly to the database, contains the documents as well as its name.
///
/// ## Example:
/// ```rs
/// let collection = database.collection("users");
/// collection.set("CoolTomato", r#"{"name": "William Henderson"}"#);
/// ```
#[derive(Debug, PartialEq, Eq)]
pub struct Collection {
    pub name: String,
    documents: Vec<Document>,
}

/// Struct representing a document.
/// Has public fields `id` and `json`.
#[derive(Debug, PartialEq, Eq)]
pub struct Document {
    pub id: String,
    pub json: String,
}

#[derive(Debug)]
pub struct CollectionError;

impl Display for CollectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "The collection could not be created/deleted")
    }
}

impl Error for CollectionError {
    fn description(&self) -> &str {
        "The collection could not be created/deleted"
    }
}

impl Database {
    /// Instantiates a new empty database with the given name.
    /// Does not allocate memory for the collections until one is created.
    ///
    /// ## Example
    /// ```rs
    /// let mut db = Database::new("myDatabase");
    /// ```
    pub fn new(name: impl AsRef<str>) -> Self {
        Self {
            collections: Vec::new(),
            name: name.as_ref().to_string(),
            writes: 0,
        }
    }

    /// Returns a reference to the named collection if it exists, or `None` otherwise.
    /// Has a time complexity of O(n) where n is the number of collections.
    pub fn collection(&self, name: impl AsRef<str>) -> Option<&Collection> {
        self.collections.iter().find(|x| x.name == name.as_ref())
    }

    /// Returns a mutable reference to the named collection if it exists, or `None` otherwise.
    /// Has a time complexity of O(n) where n is the number of collections.
    pub fn collection_mut(&mut self, name: impl AsRef<str>) -> Option<&mut Collection> {
        self.collections
            .iter_mut()
            .find(|x| x.name == name.as_ref())
    }

    /// Creates a new collection in the database with the given name.
    /// Does not allocate memory for the documents until one is created.
    /// If a collection with the same name already exists, throws `CollectionError`.
    pub fn create_collection(&mut self, name: impl AsRef<str>) -> Result<(), CollectionError> {
        if self.collections.iter().any(|x| x.name == name.as_ref()) {
            Err(CollectionError)
        } else {
            self.collections.push(Collection {
                name: name.as_ref().to_string(),
                documents: Vec::new(),
            });
            Ok(())
        }
    }

    /// Deletes a collection from the database with the given name.
    /// If the collection does not exist, throws `CollectionError`.
    pub fn delete_collection(&mut self, name: impl AsRef<str>) -> Result<(), CollectionError> {
        if let Some(collection_index) = self
            .collections
            .iter()
            .position(|x| x.name == name.as_ref())
        {
            self.collections.remove(collection_index);
            Ok(())
        } else {
            Err(CollectionError)
        }
    }

    /// Returns a reference to the internal collections Vec.
    pub fn get_collections(&self) -> &Vec<Collection> {
        &self.collections
    }

    /// Returns a reference to the internal name of the database.
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// Returns a reference to the number of writes on the database.
    pub fn get_writes(&self) -> &u64 {
        &self.writes
    }

    /// Increments the number of writes on the database by one.
    pub fn increment_writes(&mut self) {
        self.writes += 1;
    }
}

impl Collection {
    /// Gets a document from the collection with the given ID.
    /// If no document exists, returns `None`.
    pub fn get(&self, id: impl AsRef<str>) -> Option<&Document> {
        self.documents.iter().find(|x| x.id == id.as_ref())
    }

    /// Sets a document to the given value.
    /// If the JSON is invalid, returns `false`.
    /// If the document was successfully set, returns `true`.
    /// If the document already exists, it is overwritten.
    pub fn set(&mut self, id: impl AsRef<str>, value: impl AsRef<str>) -> bool {
        let new_document = Document::new(id.as_ref().to_string(), value.as_ref().to_string());

        if let Some(document) = new_document {
            if let Some(index) = self.documents.iter().position(|x| x.id == id.as_ref()) {
                self.documents.remove(index);
            }

            self.documents.push(document);

            true
        } else {
            false
        }
    }

    /// Pushes a document to the end of the collection with a random chronological ID.
    /// If the JSON is invalid, returns `false`.
    /// If the document was successfully pushed, returns `true`.
    pub fn push(&mut self, value: impl AsRef<str>) -> bool {
        let id = generate_id();

        self.set(id, value)
    }

    /// Removes a document from the collection.
    /// If the document was successfully removed, returns `true`.
    pub fn remove(&mut self, id: impl AsRef<str>) -> bool {
        if let Some(index) = self.documents.iter().position(|x| x.id == id.as_ref()) {
            self.documents.remove(index);
            true
        } else {
            false
        }
    }

    /// Lists the documents in the collection.
    pub fn list(&self) -> &Vec<Document> {
        &self.documents
    }
}

impl Document {
    /// Creates a new document object.
    /// If the JSON value is invalid, returns `None`.
    #[cfg(feature = "validation")]
    pub fn new(id: impl AsRef<str>, json: impl AsRef<str>) -> Option<Self> {
        let valid = from_str::<Value>(json.as_ref()).is_ok();
        if valid {
            Some(Self {
                id: id.as_ref().to_string(),
                json: json.as_ref().to_string(),
            })
        } else {
            None
        }
    }

    /// Creates a new document object.
    /// If the JSON value is invalid, returns `None`.
    #[cfg(not(feature = "validation"))]
    pub fn new(id: impl AsRef<str>, json: impl AsRef<str>) -> Option<Self> {
        Some(Self {
            id: id.as_ref().to_string(),
            json: json.as_ref().to_string(),
        })
    }
}
