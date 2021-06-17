use crate::{cli::LogConfig, database::Database};
use parking_lot::RwLock;
use std::{
    convert::TryInto,
    fs::File,
    io::{Read, Seek, SeekFrom},
    sync::{
        atomic::{AtomicU8, Ordering},
        Arc,
    },
    time::Duration,
};
use tar::{Archive, Builder, Header};

#[derive(Debug)]
struct Index {
    name: String,
    start: u64,
    length: u64,
}

#[derive(Debug)]
struct ISAMError;

impl std::fmt::Display for ISAMError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "An ISAM error occurred")
    }
}

impl std::error::Error for ISAMError {}

/// Loads a database from the specified file into memory using ISAM.
/// The filename should not include the `.jdb` extension.
/// This includes every document, so for large databases it could take a second.
/// Executed on program start-up.
///
/// ## Example:
/// ```rs
/// let mut db = isam::load("myDatabase").unwrap();
/// ```
pub fn load(filename: &str) -> Result<Database, Box<dyn std::error::Error>> {
    // Open the file and load the TAR archive
    let file = File::open(format!("{}.jdb", filename))?;
    let mut raw_file = File::open(format!("{}.jdb", filename))?;
    let mut archive = Archive::new(file);

    // Initialise the database object
    let mut database = Database::new(filename);

    let mut is_index = true;
    let mut indexes: Vec<Index> = Vec::new();

    // Iterate over the files in the archive
    for entry_result in archive.entries()? {
        let mut entry = entry_result?;
        let path = entry.path()?;
        let name = path
            .file_name()
            .ok_or(ISAMError {})?
            .to_str()
            .ok_or(ISAMError {})?;

        if is_index {
            // If the file is an index file, load the indexes for when reading the corresponding data file
            database.create_collection(&name[6..])?; // removes "INDEX_" prefix from index file

            let mut end_of_file = false;
            while !end_of_file {
                let mut buf: [u8; 80] = [0; 80]; // Read 80 bytes from the file

                if let Ok(()) = entry.read_exact(&mut buf) {
                    let mut document_name = String::with_capacity(64);
                    let pointer = u64::from_be_bytes(buf[64..72].try_into()?);
                    let length = u64::from_be_bytes(buf[72..80].try_into()?);

                    for ascii_char in &buf[0..64] {
                        if *ascii_char == 0 {
                            break;
                        } else {
                            document_name.push(*ascii_char as char);
                        }
                    }

                    indexes.push(Index {
                        name: document_name,
                        start: pointer,
                        length,
                    });
                } else {
                    end_of_file = true;
                };
            }
        } else {
            // If the file is a data file, load the cached indexes

            let entry_offset = entry.raw_file_position();
            for index in indexes {
                let mut buf: Vec<u8> = vec![0; index.length as usize];
                raw_file.seek(SeekFrom::Start(entry_offset + index.start))?;
                raw_file.read_exact(&mut buf)?;

                let data = std::str::from_utf8(&buf)?;

                // Add the data to the database
                database
                    .collection_mut(&name[5..])
                    .ok_or(ISAMError {})?
                    .set(&index.name, data.to_string());
            }

            indexes = Vec::new();
        }

        is_index = !is_index;
    }

    Ok(database)
}

/// Saves the given database's contents to the disk using ISAM.
/// Uses the specified filename.
///
/// ## Example:
/// ```rs
/// let mut db = Database::new("myDatabase");
/// db.create_collection("users");
/// db.collection("users").set("CoolTomato", r#"{"name": "William Henderson"}"#);
/// isam::save("myDatabase", &db);
/// ```
pub fn save(filename: &str, database: &Database) {
    let file = File::create(format!("{}.jdb", filename)).unwrap();
    let mut archive = Builder::new(file);

    for collection in database.get_collections() {
        let mut index_bytes: Vec<u8> = Vec::new();
        let mut data_bytes: Vec<u8> = Vec::new();

        for document in collection.list() {
            let document_name = document.id.as_bytes();
            let mut document_name_bytes: [u8; 64] = [0; 64];
            document_name_bytes[..document_name.len()].copy_from_slice(document_name);

            let pointer: [u8; 8] = (data_bytes.len() as u64).to_be_bytes();
            let length: [u8; 8] = (document.json.len() as u64).to_be_bytes();

            index_bytes.extend(&document_name_bytes);
            index_bytes.extend(&pointer);
            index_bytes.extend(&length);

            data_bytes.extend(document.json.as_bytes());
        }

        let mut index_header = Header::new_gnu();
        index_header.set_size(index_bytes.len() as u64);
        index_header.set_cksum();

        archive
            .append_data(
                &mut index_header,
                format!("INDEX_{}", collection.name),
                &*index_bytes,
            )
            .unwrap();

        let mut data_header = Header::new_gnu();
        data_header.set_size(data_bytes.len() as u64);
        data_header.set_cksum();

        archive
            .append_data(
                &mut data_header,
                format!("DATA_{}", collection.name),
                &*data_bytes,
            )
            .unwrap();
    }

    archive.finish().unwrap();
}

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
    crate::cli::log("[DISK] Saved to disk.", &config);

    state.store(2, Ordering::SeqCst);
}
