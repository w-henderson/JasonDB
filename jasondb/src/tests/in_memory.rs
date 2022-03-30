use crate::sources::{InMemory, Source};

#[test]
fn read_write() {
    let mut database = InMemory::new();

    let index_1 = database.write_entry("key1", "this is a value").unwrap();
    let index_2 = database.write_entry("key2", "value 2").unwrap();

    let value_1 = database.read_entry(index_1).unwrap();
    let value_2 = database.read_entry(index_2).unwrap();

    assert_eq!(value_1, "this is a value".as_bytes());
    assert_eq!(value_2, "value 2".as_bytes());

    assert!(database.read_entry(index_1 + 1).is_err());
    assert!(database.read_entry(1234).is_err());
}

#[test]
fn load_indexes() {
    let mut database = InMemory::new();

    let _ = database.write_entry("key1", "this is a value").unwrap();
    let index_2 = database.write_entry("key2", "value 2").unwrap();
    let index_3 = database.write_entry("key1", "overwritten!").unwrap();

    let indexes = database.load_indexes().unwrap();

    assert_eq!(indexes.len(), 2);
    assert_eq!(indexes["key1"], index_3);
    assert_eq!(indexes["key2"], index_2);
}

#[test]
fn compact() {
    let mut database = InMemory::new();

    database.write_entry("key1", "this is a value").unwrap();
    database.write_entry("key2", "value 2").unwrap();
    database.write_entry("key1", "overwritten!").unwrap();

    let indexes = database.load_indexes().unwrap();

    database.compact(&indexes).unwrap();

    assert!(
        database.data == b"\x04\0\0\0\0\0\0\0key2\x07\0\0\0\0\0\0\0value 2\x04\0\0\0\0\0\0\0key1\x0c\0\0\0\0\0\0\0overwritten!" ||
        database.data == b"\x04\0\0\0\0\0\0\0key1\x0c\0\0\0\0\0\0\0overwritten!\x04\0\0\0\0\0\0\0key2\x07\0\0\0\0\0\0\0value 2"
    );
}
