#![allow(unused_imports)]
use crate::request;
use crate::Database;
use parking_lot::RwLock;
use std::sync::Arc;

#[allow(dead_code)]
fn init_database() -> Arc<RwLock<Database>> {
    // Set up a thread-safe database instance (simulating the real program)
    let database = Arc::new(RwLock::new(Database::new("test")));
    let mut db = database.write();
    (*db).create_collection("users").unwrap();

    // Add some example data to the database
    let users = (*db).collection_mut("users").unwrap();
    users.set("CoolTomato", r#"{"name": "William Henderson"}"#.to_string());
    users.set("Chrome599", r#"{"name": "Frankie Lambert"}"#.to_string());

    // Drop the reference to the database
    drop(db);

    database
}

#[test]
fn test_successful_get() {
    let database = init_database();

    // Create and attempt to parse the command
    let command = "GET CoolTomato FROM users";
    let request = request::parse(command);
    let expected_request = request::Request::Get {
        collection: "users",
        document: "CoolTomato",
    };

    // Assert that the command was parsed correctly
    assert_eq!(request, expected_request);

    // Attempt to execute the command
    let response = request::execute(request, &database);
    let expected_response = request::Response::Success {
        data: Some(r#"{"name": "William Henderson"}"#.to_string()),
    };

    // Assert that the response was correct
    assert_eq!(response, expected_response);
}

#[test]
fn test_successful_set() {
    let database = init_database();

    // Create and attempt to parse the command
    let command = r#"SET flauntingspade4 FROM users TO {"name": "Elliot Whybrow"}"#;
    let request = request::parse(command);
    let expected_request = request::Request::Set {
        collection: "users",
        document: "flauntingspade4",
        value: r#"{"name": "Elliot Whybrow"}"#.to_string(),
    };

    // Assert that the command was parsed correctly
    assert_eq!(request, expected_request);

    // Attempt to execute the command
    let response = request::execute(request, &database);
    let expected_response = request::Response::Success { data: None };

    // Assert that the response was correct
    assert_eq!(response, expected_response);

    // Assert that the data was correctly set
    let db = database.read();
    let new_data = &(*db)
        .collection("users")
        .unwrap()
        .get("flauntingspade4")
        .unwrap()
        .json;
    assert_eq!(new_data, r#"{"name": "Elliot Whybrow"}"#);
}

#[test]
fn test_successful_create() {
    let database = init_database();

    // Create and attempt to parse the command
    let command = "CREATE messages";
    let request = request::parse(command);
    let expected_request = request::Request::Create {
        collection: "messages",
    };

    // Assert that the command was parsed correctly
    assert_eq!(request, expected_request);

    // Attempt to execute the command
    let response = request::execute(request, &database);
    let expected_response = request::Response::Success { data: None };

    // Assert that the response was correct
    assert_eq!(response, expected_response);

    // Assert that the collection was successfully created
    let db = database.read();
    assert!((*db).collection("messages").is_some());
}

#[test]
fn test_successful_list() {
    let database = init_database();

    // Create and attempt to parse the command
    let command = "LIST users";
    let request = request::parse(command);
    let expected_request = request::Request::List {
        collection: "users",
        condition: None,
    };

    // Assert that the command was parsed correctly
    assert_eq!(request, expected_request);

    // Attempt to execute the command
    let response = request::execute(request, &database);
    let expected_response = request::Response::Success {
        data: Some(
            r#"[{"id": "CoolTomato", "data": {"name": "William Henderson"}}, {"id": "Chrome599", "data": {"name": "Frankie Lambert"}}]"#.to_string(),
        ),
    };

    // Assert that the response was correct
    assert_eq!(response, expected_response);
}

#[test]
fn test_successful_delete() {
    let database = init_database();

    // Create and attempt to parse the command
    let command = "DELETE users";
    let request = request::parse(command);
    let expected_request = request::Request::Delete {
        collection: "users",
    };

    // Assert that the command was parsed correctly
    assert_eq!(request, expected_request);

    // Attempt to execute the command
    let response = request::execute(request, &database);
    let expected_response = request::Response::Success { data: None };

    // Assert that the response was correct
    assert_eq!(response, expected_response);

    // Assert that the collection was successfully deleted
    let db = database.read();
    assert!((*db).collection("users").is_none());
}
