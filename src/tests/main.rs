#![allow(unused_imports)]
use crate::{database::Database, isam};
use std::fs::remove_file;

#[test]
/// Tests whether the program can create a database, add data to it, save it to disk, and read it back from disk.
/// This partially tests the `database` module and fully tests the `isam` module.
fn create_save_load() {
    // Create a database and fill it with example data
    let mut db = Database::new("test");
    db.create_collection("users").unwrap();
    let users = db.collection_mut("users").unwrap();
    users.set("CoolTomato", r#"{"name": "William Henderson"}"#.to_string());
    users.set("Chrome599", r#"{"name": "Frankie Lambert"}"#.to_string());

    // Save the database using ISAM
    isam::save(&db);

    // Load the database back again using ISAM
    let new_db = isam::load("test");

    // Assert that the original in-memory instance is identical to that loaded from disk
    assert_eq!(db, new_db);
}
