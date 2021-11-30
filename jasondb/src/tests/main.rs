use crate::database::Database;
use crate::isam;
use crate::prelude::*;

#[test]
/// Tests whether the program can create a database, add data to it, save it to disk, and read it back from disk.
/// This partially tests the `database` module and fully tests the `isam` module.
fn create_save_load() {
    // Create a database and fill it with example data
    let mut db = Database::new("test.jdb");
    set!(&mut db, "users/user1", "{\"name\": \"William Henderson\"}");
    set!(&mut db, "users/user2", "{\"name\": \"Frankie Lambert\"}");

    // Save the database using ISAM
    isam::save("test.jdb", &db);

    // Load the database back again using ISAM
    let new_db = isam::load("test.jdb").unwrap();

    // Assert that the original in-memory instance is identical to that loaded from disk
    assert_eq!(db, new_db);
}
