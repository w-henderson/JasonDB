use crate::database::{Collection, Database, Document};

/// Represents a database component that is readable at a path.
///
/// Not intended to be used directly, but rather through the macros.
pub trait DatabaseReadable {
    fn read(&self, path: &str) -> &Document;
}

impl DatabaseReadable for Database {
    fn read(&self, path: &str) -> &Document {
        let mut path_parts = path.split('/');
        let collection_id = path_parts.next().unwrap();
        let document_id = path_parts.next().unwrap();

        let collection = self.collection(collection_id).unwrap();
        let document = collection.get(document_id).unwrap();

        document
    }
}

impl DatabaseReadable for Collection {
    fn read(&self, path: &str) -> &Document {
        self.get(path).unwrap()
    }
}
/// Represents a database component that is writable at a path.
///
/// Not intended to be used directly, but rather through the macros.
pub trait DatabaseWritable<K, V>
where
    K: AsRef<str>,
    V: AsRef<str>,
{
    fn write(&mut self, path: K, value: V);
}

impl<K, V> DatabaseWritable<K, V> for Database
where
    K: AsRef<str>,
    V: AsRef<str>,
{
    fn write(&mut self, path: K, value: V) {
        let mut path_parts = path.as_ref().split('/');
        let collection_id = path_parts.next().unwrap();
        let document_id = path_parts.next().unwrap();

        let collection = if let Some(collection) = self.collection_mut(collection_id) {
            collection
        } else {
            self.create_collection(collection_id).unwrap();
            self.collection_mut(collection_id).unwrap()
        };

        collection.set(document_id, value);
    }
}

impl<K, V> DatabaseWritable<K, V> for Collection
where
    K: AsRef<str>,
    V: AsRef<str>,
{
    fn write(&mut self, path: K, value: V) {
        self.set(path, value);
    }
}

/// Return a reference to the document at the given path.
///
/// The first argument (the "root") can be either a reference to a `Database` or a reference to a `Collection`.
/// The second argument is the path from the root to the document.
///
/// ## Examples
/// ```
/// // These do the same thing.
/// let doc = document!(&db, "users/w-henderson"); // path relative to the database
/// let doc = document!(&users, "w-henderson"); // path relative to the collection
/// ```
///
/// ## Panics
/// This macro will panic if the path is invalid. If you need to handle these cases,
///   use the regular methods on the `Database` struct instead.
#[macro_export]
macro_rules! document {
    ($root:expr, $path:expr) => {
        DatabaseReadable::read($root, $path)
    };
}

/// Return a reference to the collection at the given path.
///
/// ## Examples
/// ```
/// let users = collection!(&db, "users");
/// ```
///
/// ## Panics
/// This macro will panic if the path is invalid. If you need to handle these cases,
///   use the regular methods on the `Database` struct instead.
#[macro_export]
macro_rules! collection {
    ($root:expr, $path:expr) => {
        $root.collection($path).unwrap()
    };
}

/// Return a mutable reference to the collection at the given path.
///
/// ## Examples
/// ```
/// let users = collection_mut!(&mut db, "users");
/// ```
///
/// ## Panics
/// This macro will panic if the path is invalid. If you need to handle these cases,
///   use the regular methods on the `Database` struct instead.
#[macro_export]
macro_rules! collection_mut {
    ($root:expr, $path:expr) => {
        $root.collection_mut($path).unwrap()
    };
}

/// Set a document at the given path to the given value.
///
/// - If the document does not exist, it will be created.
/// - If the collection does not exist, it will be created.
///
/// The first argument (the "root") can be either a mutable reference to a `Database` or to a `Collection`.
/// The second argument is the path from the root to the document.
///
/// ## Examples
/// ```
/// // These do the same thing.
/// set!(&mut db, "users/w-henderson", "{\"name\": \"William Henderson\"}"); // path relative to the database
/// set!(&mut users, "w-henderson", "{\"name\": \"William Henderson\"}"); // path relative to the collection
/// ```
#[macro_export]
macro_rules! set {
    ($root:expr, $path:expr, $value:expr) => {
        DatabaseWritable::write($root, $path, $value);
    };
}

/// Pushes the given value to the collection at the given path.
///
/// ## Examples
/// ```
/// push!(&mut db, "messages", "{\"text\": \"Hello, world!\"}");
/// ```
///
/// ## Panics
/// This macro will panic if the path is invalid. If you need to handle these cases,
///  use the regular methods on the `Database` struct instead.
#[macro_export]
macro_rules! push {
    ($root:expr, $path:expr, $value:expr) => {
        $root.collection_mut($path).unwrap().push($value);
    };
}
