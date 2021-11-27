use crate::cli::{log, LogConfig};

use jasondb::database::Database;
use jasondb::isam::save;

use parking_lot::RwLock;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// Handles mirroring the database to the disk.
/// Updates the disk every <interval> seconds if the database has changed.
pub async fn mirror_handler(
    database: Arc<RwLock<Database>>,
    filename: &str,
    interval: u64,
    state: Arc<AtomicU8>,
    config: LogConfig,
) {
    let mut cached_writes: u64 = 0;

    while state.load(Ordering::SeqCst) == 0 {
        let db = database.read();
        let new_writes = db.get_writes();

        if new_writes > &cached_writes {
            cached_writes = *new_writes;
            save(filename, &*db);
            crate::cli::log("[DISK] Saved to disk.", &config);
        }

        drop(db);
        std::thread::park_timeout(Duration::from_secs(interval));
    }

    let db = database.read();
    save(filename, &*db);
    log("[DISK] Saved to disk.", &config);

    state.store(2, Ordering::SeqCst);
}
