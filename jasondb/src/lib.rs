//! # JasonDB
//! JasonDB is a NoSQL, document-oriented, JSON-based database management system built with the modern web in mind. It is fast, flexible, and easy-to-use, making it a solid choice for building databases for web applications. It also provides a number of macros allowing for powerful operations in concise syntax.
//!
//! ## Installation
//! The JasonDB crate can be installed by adding `jasondb` to your `Cargo.toml` file.
//!
//! ## Documentation
//! The JasonDB documentation can be found at [docs.rs](https://docs.rs/jasondb).
//!
//! ## Basic Example
//! ```rs
//! use jasondb::JasonDB;
//! use jasondb::prelude::*;
//!
//! fn main() {
//!     // Create a new database (use `JasonDB::open` to open an existing database)
//!     let db = JasonDB::new("/path/to/database.jdb");
//!
//!     // Lock the database for writing, then write to it
//!     // We do this in a new scope so the database is unlocked as soon as we're done
//!     {
//!         let mut db_write = db.write();
//!         set!(&mut db_write, "users/w-henderson", "{\"name\": \"William Henderson\"}");
//!         set!(&mut db_write, "users/torvalds", "{\"name\": \"Linus Torvalds\"}");
//!     }
//!
//!     // Lock the database for reading, then read from it
//!     // Note that this is a contrived example (one could read from the write-locked database above)
//!     {
//!         let db_read = db.read();
//!         let test = get!(&db_read, "users/w-henderson");
//!         assert_eq!(test.json, "{\"name\": \"William Henderson\"}");
//!     }
//! }
//! ```
//!
//! ## Further Examples
//! - [Message Board Example](https://github.com/w-henderson/Humphrey/tree/master/examples/database): A simple example of integrating JasonDB with a web application.

pub mod database;
pub mod isam;
pub mod macros;

mod id;
mod tar;

/// Re-exports macros and the traits required to use them.
pub mod prelude {
    pub use crate::macros::*;
    pub use crate::{collection, collection_mut, document, push, set};
}

#[cfg(test)]
mod tests;

pub use crate::database::Database;

use std::error::Error;
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::thread::{park_timeout, spawn, JoinHandle};
use std::time::Duration;

/// High-level abstraction over the database.
/// Handles multi-threading and disk synchronization behind the scenes.
///
/// ## Examples
/// ```
/// use jasondb::JasonDB;
/// use jasondb::prelude::*; // only necessary if using macros
///
/// // Create a new database (use `JasonDB::open` to open an existing database)
/// let db = JasonDB::new("/path/to/database.jdb");
///
/// // Lock the database for writing, then write to it
/// // We do this in a new scope so the database is unlocked as soon as we're done
/// {
///     let mut db_write = db.write();
///     set!(&mut db_write, "users/w-henderson", "{\"name\": \"William Henderson\"}");
///     set!(&mut db_write, "users/torvalds", "{\"name\": \"Linus Torvalds\"}");
/// }
///
/// // Lock the database for reading, then read from it
/// // Note that this is a contrived example (one could read from the write-locked database above)
/// {
///     let db_read = db.read();
///     let test = get!(&db_read, "users/w-henderson");
///     assert_eq!(test.json, "{\"name\": \"William Henderson\"}");
/// }
/// ```
pub struct JasonDB {
    database: Arc<RwLock<Database>>,
    isam_thread: Option<JoinHandle<()>>,
    isam_thread_channel: SyncSender<u8>,
}

impl JasonDB {
    /// Create a new database and store it at the given path.
    ///
    /// This starts a background thread that will periodically copy the database to disk.
    /// The thread will run until the database is dropped, at which point it will be gracefully stopped.
    pub fn new(filename: &'static str) -> Self {
        let database = Database::new(filename);
        Self::init(database, filename)
    }

    /// Open an existing database at the given path.
    /// Returns an error if the database cannot be opened or read.
    ///
    /// This starts a background thread that will periodically copy the database to disk.
    /// The thread will run until the database is dropped, at which point it will be gracefully stopped.
    pub fn open(filename: &'static str) -> Result<Self, Box<dyn Error>> {
        let database = isam::load(filename)?;
        Ok(Self::init(database, filename))
    }

    /// Locks the database's `RwLock` for reading.
    /// This blocks until the lock can be acquired.
    pub fn read(&self) -> RwLockReadGuard<Database> {
        self.database.read().unwrap()
    }

    /// Locks the database's `RwLock` for writing.
    /// This blocks until the lock can be acquired.
    pub fn write(&self) -> RwLockWriteGuard<Database> {
        self.database.write().unwrap()
    }

    /// Initialises a new `JasonDB` instance from an already-instantiated `Database` and a filename.
    /// Equivalent to `JasonDB::new` but with a pre-existing `Database` instance.
    ///
    /// This starts a background thread that will periodically copy the database to disk.
    /// The thread will run until the database is dropped, at which point it will be gracefully stopped.
    pub fn init(database: Database, filename: &'static str) -> Self {
        let database = Arc::new(RwLock::new(database));

        let isam_database_ref = database.clone();
        let (isam_tx, isam_rx) = sync_channel(1);
        let isam_thread = spawn(move || isam_thread(filename, isam_database_ref, isam_rx));

        Self {
            database,
            isam_thread: Some(isam_thread),
            isam_thread_channel: isam_tx,
        }
    }

    /// Stops the background ISAM thread.
    fn stop_isam_thread(&mut self) {
        if let Some(thread) = self.isam_thread.take() {
            self.isam_thread_channel.send(1).unwrap();
            thread.thread().unpark();

            thread.join().unwrap();
        }
    }
}

impl Drop for JasonDB {
    fn drop(&mut self) {
        self.stop_isam_thread();
    }
}

/// A function that periodically copies the database to disk.
/// This function will return once a message is received on the channel.
fn isam_thread(filename: &'static str, database: Arc<RwLock<Database>>, recv: Receiver<u8>) {
    loop {
        {
            isam::save(filename, &database.read().unwrap());
        }
        park_timeout(Duration::from_secs(1));

        if recv.try_recv().is_ok() {
            return;
        }
    }
}
