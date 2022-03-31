use crate::error::JasonError;
use crate::sources::{InMemory, Source};
use crate::Database;

use crate::tests::mock::Person;

#[test]
fn basic() -> Result<(), JasonError> {
    let source = InMemory::new();
    let mut database: Database<Person, InMemory> = Database::new(source)?;
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

    let old_len = database.source.data.len();

    database.source.compact(&database.indexes)?;
    database.indexes = database.source.load_indexes()?;

    assert_eq!(database.iter().count(), 3);
    assert!(database.source.data.len() < old_len);

    Ok(())
}

#[test]
fn delete() -> Result<(), JasonError> {
    let source = InMemory::new();
    let mut database: Database<Person, InMemory> = Database::new(source)?;

    let person_1 = Person::new("Elizabeth II", 1926);
    database.set("queen_elizabeth_ii", &person_1)?;
    database.delete("queen_elizabeth_ii")?;

    assert_eq!(database.iter().count(), 0);
    assert!(!database.source.data.is_empty());

    database.source.compact(&database.indexes)?;
    database.indexes = database.source.load_indexes()?;

    assert_eq!(database.iter().count(), 0);
    assert_eq!(database.source.data.len(), 0);

    Ok(())
}

#[test]
fn conditional_query() -> Result<(), JasonError> {
    let source = InMemory::new();
    let mut database: Database<Person, InMemory> = Database::new(source)?;

    let person_1 = Person::new("Johann Sebastian Bach", 1685);
    let person_2 = Person::new("Wolfgang Amadeus Mozart", 1756);
    let person_3 = Person::new("Johannes Brahms", 1833);
    let person_4 = Person::new("Camille Saint-Saëns", 1835);
    let person_5 = Person::new("Pyotr Ilyich Tchaikovsky", 1840);
    let person_6 = Person::new("Dmitri Shostakovich", 1906);

    database.set("bach", &person_1)?;
    database.set("mozart", &person_2)?;
    database.set("brahms", &person_3)?;
    database.set("saint_saens", &person_4)?;
    database.set("tchaikovsky", &person_5)?;
    database.set("shostakovich", &person_6)?;

    // Get only 19th-century composers
    let composers: Vec<String> = database
        .iter()
        .flatten()
        .filter(|(_, person)| person.year_of_birth >= 1800 && person.year_of_birth < 1900)
        .map(|(_, person)| person.name)
        .collect();

    assert_eq!(composers.len(), 3);
    assert!(composers.contains(&"Johannes Brahms".to_string()));
    assert!(composers.contains(&"Camille Saint-Saëns".to_string()));
    assert!(composers.contains(&"Pyotr Ilyich Tchaikovsky".to_string()));

    Ok(())
}
