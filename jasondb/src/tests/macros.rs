use crate::database::Database;
use crate::prelude::*;

fn set_up_db() -> Database {
    let mut db = Database::new("test.jdb");
    db.create_collection("users").unwrap();

    let users = db.collection_mut("users").unwrap();
    users.set("w-henderson", "{\"name\": \"William Henderson\"}".into());

    db
}

#[test]
fn test_read_macros() {
    let db = set_up_db();

    // Use macros to read the document in a number of ways
    let macro_collection = collection!(&db, "users");
    let macro_document = document!(&db, "users/w-henderson");
    let macro_document_2 = document!(macro_collection, "w-henderson");

    let expected_json = "{\"name\": \"William Henderson\"}".to_string();
    let expected_collection = db.collection("users").unwrap();

    // Check that the macros return the expected values
    assert_eq!(macro_document.json, expected_json);
    assert_eq!(macro_document_2.json, expected_json);
    assert_eq!(macro_collection, expected_collection);
}

#[test]
fn test_write_macros() {
    let mut db = set_up_db();

    // Check that the database is set up correctly
    assert_eq!(db.collection("users").unwrap().list().len(), 1);
    assert!(db.collection("admins").is_none());

    let expected_json_1 = "{\"name\": \"Alice\"}";
    let expected_json_2 = "{\"name\": \"Bob\"}";

    // Set some documents using macros
    set!(&mut db, "users/alice", expected_json_1.into());
    set!(&mut db, "admins/alice", expected_json_1.into()); // IMPORTANT: this will create the collection

    // Set some documents in a collection using the macro again
    let collection = collection_mut!(&mut db, "users");
    set!(collection, "bob", expected_json_2.into());

    // Check that the macros set the values correctly
    let resulting_json_1 = &db.collection("users").unwrap().get("alice").unwrap().json;
    let resulting_json_2 = &db.collection("users").unwrap().get("bob").unwrap().json;
    let resulting_json_3 = &db.collection("admins").unwrap().get("alice").unwrap().json;

    assert_eq!(resulting_json_1, expected_json_1);
    assert_eq!(resulting_json_2, expected_json_2);
    assert_eq!(resulting_json_3, expected_json_1);
}
