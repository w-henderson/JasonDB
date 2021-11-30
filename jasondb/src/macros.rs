use crate::database::{Collection, Database, Document};

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

pub trait DatabaseWritable {
    fn write(&mut self, path: &str, value: String);
}

impl DatabaseWritable for Database {
    fn write(&mut self, path: &str, value: String) {
        let mut path_parts = path.split('/');
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

impl DatabaseWritable for Collection {
    fn write(&mut self, path: &str, value: String) {
        self.set(path, value);
    }
}

#[macro_export]
macro_rules! document {
    ($root:expr, $path:expr) => {
        DatabaseReadable::read($root, $path)
    };
}

#[macro_export]
macro_rules! collection {
    ($root:expr, $path:expr) => {
        $root.collection($path).unwrap()
    };
}

#[macro_export]
macro_rules! collection_mut {
    ($root:expr, $path:expr) => {
        $root.collection_mut($path).unwrap()
    };
}

#[macro_export]
macro_rules! set {
    ($root:expr, $path:expr, $value:expr) => {
        DatabaseWritable::write($root, $path, $value);
    };
}
