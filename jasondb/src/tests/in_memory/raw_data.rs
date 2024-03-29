use crate::sources::{InMemory, Source};

use humphrey_json::prelude::*;

#[test]
fn read_write() {
    let mut database = InMemory::new();

    let index_1 = database.write_entry("key1", "this is a value").unwrap();
    let index_2 = database.write_entry("key2", "value 2").unwrap();

    let value_1 = database.read_entry(index_1).unwrap();
    let value_2 = database.read_entry(index_2).unwrap();

    assert_eq!(value_1, ("key1".to_string(), b"this is a value".to_vec()));
    assert_eq!(value_2, ("key2".to_string(), b"value 2".to_vec()));

    assert!(database.read_entry(index_1 + 1).is_err());
    assert!(database.read_entry(1234).is_err());
}

#[test]
fn load_indexes() {
    let mut database = InMemory::new();

    database.write_entry("key1", "this is a value").unwrap();
    let index_2 = database.write_entry("key2", "value 2").unwrap();
    let index_3 = database.write_entry("key1", "overwritten!").unwrap();
    database.write_entry("key3", "not null").unwrap();
    database.write_entry("key3", "null").unwrap();

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
    database.write_entry("key3", "not null").unwrap();
    database.write_entry("key3", "null").unwrap();

    let indexes = database.load_indexes().unwrap();

    database.compact(&indexes).unwrap();

    assert!(
        database.data == b"\x04\0\0\0\0\0\0\0key2\x07\0\0\0\0\0\0\0value 2\x04\0\0\0\0\0\0\0key1\x0c\0\0\0\0\0\0\0overwritten!" ||
        database.data == b"\x04\0\0\0\0\0\0\0key1\x0c\0\0\0\0\0\0\0overwritten!\x04\0\0\0\0\0\0\0key2\x07\0\0\0\0\0\0\0value 2"
    );
}

#[test]
fn index_on() -> Result<(), Box<dyn std::error::Error>> {
    let mut database = InMemory::new();

    let elizabeth_ii = database.write_entry(
        "elizabeth_ii",
        json!({"name": "Elizabeth II", "year_of_birth": 1926, "gender": "female"}).serialize(),
    )?;

    let george_vi = database.write_entry(
        "george_vi",
        json!({"name": "George VI", "year_of_birth": 1895, "gender": "male"}).serialize(),
    )?;

    let edward_viii = database.write_entry(
        "edward_viii",
        json!({"name": "Edward VIII", "year_of_birth": 1894, "gender": "male"}).serialize(),
    )?;

    let indexes = database.load_indexes()?;
    let index_on_gender = database.index_on("gender", &indexes)?;
    let index_on_year = database.index_on("year_of_birth", &indexes)?;

    let men = index_on_gender.get(&json!("male")).unwrap();
    assert_eq!(men.len(), 2);
    assert!(men.contains(&george_vi));
    assert!(men.contains(&edward_viii));
    assert!(!men.contains(&elizabeth_ii));

    let women = index_on_gender.get(&json!("female")).unwrap();
    assert_eq!(*women, [elizabeth_ii].iter().cloned().collect());

    let born_in_1895: &std::collections::BTreeSet<u64> = index_on_year.get(&json!(1895)).unwrap();
    assert_eq!(*born_in_1895, [george_vi].iter().cloned().collect());

    let born_in_1900 = index_on_year.get(&json!(1900));
    assert!(born_in_1900.is_none());

    Ok(())
}
