pub mod database;
pub mod isam;

#[cfg(test)]
mod tests;

use crate::database::Database;

use std::error::Error;
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::thread::{park_timeout, spawn, JoinHandle};
use std::time::Duration;

pub struct JasonDB {
    database: Arc<RwLock<Database>>,
    isam_thread: Option<JoinHandle<()>>,
    isam_thread_channel: SyncSender<u8>,
}

impl JasonDB {
    pub fn new(filename: &'static str) -> Self {
        let database = Database::new(filename);
        Self::init(database, filename)
    }

    pub fn open(filename: &'static str) -> Result<Self, Box<dyn Error>> {
        let database = isam::load(filename)?;
        Ok(Self::init(database, filename))
    }

    pub fn read(&self) -> RwLockReadGuard<Database> {
        self.database.read().unwrap()
    }

    pub fn write(&self) -> RwLockWriteGuard<Database> {
        self.database.write().unwrap()
    }

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
