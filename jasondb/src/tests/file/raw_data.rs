use crate::sources::{FileSource, Source};

use humphrey_json::prelude::*;

use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom, Write};

#[test]
fn read_write() {
    let mut database = FileSource::new("test_read_write.jdb").unwrap();

    let index_1 = database.write_entry("key1", "this is a value").unwrap();
    let index_2 = database.write_entry("key2", "value 2").unwrap();

    let value_1 = database.read_entry(index_1).unwrap();
    let value_2 = database.read_entry(index_2).unwrap();

    assert_eq!(value_1, "this is a value".as_bytes());
    assert_eq!(value_2, "value 2".as_bytes());

    assert!(database.read_entry(index_1 + 1).is_err());
    assert!(database.read_entry(1234).is_err());

    drop(database);
    fs::remove_file("test_read_write.jdb").unwrap();
}

#[test]
fn load_indexes() {
    let mut database = FileSource::new("test_load_indexes.jdb").unwrap();

    database.write_entry("key1", "this is a value").unwrap();
    let index_2 = database.write_entry("key2", "value 2").unwrap();
    let index_3 = database.write_entry("key1", "overwritten!").unwrap();
    database.write_entry("key3", "not null").unwrap();
    database.write_entry("key3", "null").unwrap();

    let indexes = database.load_indexes().unwrap();

    assert_eq!(indexes.len(), 2);
    assert_eq!(indexes["key1"], index_3);
    assert_eq!(indexes["key2"], index_2);

    drop(database);
    fs::remove_file("test_load_indexes.jdb").unwrap();
}

#[test]
fn compact() {
    let mut database = FileSource::new("test_compact.jdb").unwrap();

    database.write_entry("key1", "this is a value").unwrap();
    database.write_entry("key2", "value 2").unwrap();
    database.write_entry("key1", "overwritten!").unwrap();
    database.write_entry("key3", "not null").unwrap();
    database.write_entry("key3", "null").unwrap();

    let indexes = database.load_indexes().unwrap();

    database.compact(&indexes).unwrap();

    let mut buf: Vec<u8> = vec![0; database.len as usize];
    database.file.seek(SeekFrom::Start(0)).unwrap();
    database.file.read_exact(&mut buf).unwrap();
    assert!(
        buf == b"\x04\0\0\0\0\0\0\0key2\x07\0\0\0\0\0\0\0value 2\x04\0\0\0\0\0\0\0key1\x0c\0\0\0\0\0\0\0overwritten!" ||
        buf == b"\x04\0\0\0\0\0\0\0key1\x0c\0\0\0\0\0\0\0overwritten!\x04\0\0\0\0\0\0\0key2\x07\0\0\0\0\0\0\0value 2"
    );

    drop(database);
    fs::remove_file("test_compact.jdb").unwrap();
}

#[test]
fn open_existing() {
    {
        let mut file = File::create("test_open_existing.jdb").unwrap();
        file.write_all(b"\x04\0\0\0\0\0\0\0key2\x07\0\0\0\0\0\0\0value 2\x04\0\0\0\0\0\0\0key1\x0c\0\0\0\0\0\0\0overwritten!").unwrap();
    }

    let mut database = FileSource::new("test_open_existing.jdb").unwrap();

    let value_1 = database.read_entry(0).unwrap();
    let value_2 = database.read_entry(27).unwrap();

    assert_eq!(value_1, "value 2".as_bytes());
    assert_eq!(value_2, "overwritten!".as_bytes());

    drop(database);
    fs::remove_file("test_open_existing.jdb").unwrap();
}

#[test]
fn index_on() -> Result<(), Box<dyn std::error::Error>> {
    let mut database = FileSource::new("test_index_on.jdb")?;

    let elizabeth_ii = database.write_entry(
        "elizabeth_ii",
        json!({"name": "Elizabeth II", "yearOfBirth": 1926, "gender": "female"}).serialize(),
    )?;

    let george_vi = database.write_entry(
        "george_vi",
        json!({"name": "George VI", "yearOfBirth": 1895, "gender": "male"}).serialize(),
    )?;

    let edward_viii = database.write_entry(
        "edward_viii",
        json!({"name": "Edward VIII", "yearOfBirth": 1894, "gender": "male"}).serialize(),
    )?;

    let indexes = database.load_indexes()?;
    let index_on_gender = database.index_on("gender", &indexes)?;
    let index_on_year = database.index_on("yearOfBirth", &indexes)?;

    let men = index_on_gender.get(&json!("male")).unwrap();
    assert_eq!(men.len(), 2);
    assert!(men.contains(&george_vi));
    assert!(men.contains(&edward_viii));
    assert!(!men.contains(&elizabeth_ii));

    let women = index_on_gender.get(&json!("female")).unwrap();
    assert_eq!(*women, vec![elizabeth_ii]);

    let born_in_1895 = index_on_year.get(&json!(1895)).unwrap();
    assert_eq!(*born_in_1895, vec![george_vi]);

    let born_in_1900 = index_on_year.get(&json!(1900));
    assert!(born_in_1900.is_none());

    drop(database);
    fs::remove_file("test_index_on.jdb").unwrap();

    Ok(())
}
