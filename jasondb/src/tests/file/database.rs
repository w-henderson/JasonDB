use crate::error::JasonError;
use crate::sources::FileSource;
use crate::Database;

use crate::tests::mock::Person;

use std::fs;

#[test]
fn basic() -> Result<(), JasonError> {
    let source = FileSource::create("test_db_basic.jdb")?;
    let mut database: Database<Person> = Database::new(source)?;
    assert_eq!(database.iter().count(), 0);

    let person_1 = Person::new("Elizabeth II", 1925);
    let person_2 = Person::new("George VI", 1895);
    let person_3 = Person::new("Edward VIII", 1894);

    database.set("queen_elizabeth_ii", &person_1)?;
    database.set("king_george_vi", &person_2)?;
    database.set("king_edward_viii", &person_3)?;

    let person_1 = Person::new("Elizabeth II", 1926);
    database.set("queen_elizabeth_ii", &person_1)?;

    assert_eq!(database.iter().count(), 3);
    assert_eq!(database.get("queen_elizabeth_ii"), Ok(person_1));
    assert_eq!(database.get("king_george_vi"), Ok(person_2));
    assert_eq!(database.get("king_edward_viii"), Ok(person_3));
    assert_eq!(database.get("king_george_v"), Err(JasonError::InvalidKey));

    let old_len = database.source.len;

    let source = FileSource::open("test_db_basic.jdb")?;
    let mut database: Database<Person> = Database::new(source)?;
    assert_eq!(database.iter().count(), 3);
    assert!(database.source.len < old_len);

    fs::remove_file("test_db_basic.jdb").unwrap();

    Ok(())
}
