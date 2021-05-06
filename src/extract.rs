//! Manages extraction of JDB files.

use crate::isam;
use std::{fs::File, io::Write};

/// Extracts a JDB file into a directory.
pub fn extract(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Load the database and create a directory for it
    let database = isam::load(path)?;
    std::fs::create_dir(path)?;

    // Iterate over the collections of the database
    for collection in database.get_collections() {
        std::fs::create_dir(format!("{}/{}", path, collection.name))?;
        for document in collection.list() {
            // For every document of every collection, write it to a file
            let mut file =
                File::create(format!("{}/{}/{}.json", path, collection.name, document.id))?;
            file.write(document.json.as_bytes())?;
        }
    }

    Ok(())
}
