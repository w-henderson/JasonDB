use crate::error::JasonError;
use crate::sources::{InMemory, Source};
use crate::Database;

use crate::tests::mock::{composers_db, AgedPerson, Person};

use std::fs;

#[test]
fn basic() -> Result<(), JasonError> {
    let mut database: Database<Person, InMemory> = Database::new_in_memory();
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

    database.source.compact(&database.primary_indexes)?;
    database.primary_indexes = database.source.load_indexes()?;

    assert_eq!(database.iter().count(), 3);
    assert!(database.source.data.len() < old_len);

    Ok(())
}

#[test]
fn delete() -> Result<(), JasonError> {
    let mut database: Database<Person, InMemory> = Database::new_in_memory();

    let person_1 = Person::new("Elizabeth II", 1926);
    database.set("queen_elizabeth_ii", &person_1)?;
    database.delete("queen_elizabeth_ii")?;

    assert_eq!(database.iter().count(), 0);
    assert!(!database.source.data.is_empty());

    database.source.compact(&database.primary_indexes)?;
    database.primary_indexes = database.source.load_indexes()?;

    assert_eq!(database.iter().count(), 0);
    assert_eq!(database.source.data.len(), 0);

    Ok(())
}

#[test]
fn optimised_query_1() -> Result<(), JasonError> {
    let source = InMemory::new();
    let mut database = composers_db(source)?.with_index(field!(year_of_birth))?;

    // Get only 19th-century composers
    let query = query!(year_of_birth >= 1800) & query!(year_of_birth < 1900);

    let composers: Vec<String> = query
        .execute_optimised(&mut database)?
        .flatten()
        .map(|(_, person)| person.name)
        .collect();

    assert_eq!(composers.len(), 3);
    assert!(composers.contains(&"Johannes Brahms".to_string()));
    assert!(composers.contains(&"Camille Saint-Saëns".to_string()));
    assert!(composers.contains(&"Pyotr Ilyich Tchaikovsky".to_string()));

    Ok(())
}

#[test]
fn optimised_query_2() -> Result<(), JasonError> {
    let source = InMemory::new();
    let mut database = composers_db(source)?
        .with_index(field!(name))?
        .with_index(field!(year_of_birth))?;

    // Get only 19th-century composers
    let query = query!(year_of_birth >= 1800) & query!(name == "Johannes Brahms");

    let composers: Vec<String> = query
        .execute_optimised(&mut database)?
        .flatten()
        .map(|(_, person)| person.name)
        .collect();

    assert_eq!(composers.len(), 1);
    assert!(composers.contains(&"Johannes Brahms".to_string()));

    Ok(())
}

#[test]
fn optimised_query_3() -> Result<(), JasonError> {
    let source = InMemory::new();
    let mut database = composers_db(source)?
        .with_index(field!(name))?
        .with_index(field!(year_of_birth))?;

    // Get only 19th-century composers
    let query = query!(year_of_birth >= 1900) | query!(name == "Johannes Brahms");

    let composers: Vec<String> = query
        .execute_optimised(&mut database)?
        .flatten()
        .map(|(_, person)| person.name)
        .collect();

    assert_eq!(composers.len(), 2);
    assert!(composers.contains(&"Dmitri Shostakovich".to_string()));
    assert!(composers.contains(&"Johannes Brahms".to_string()));

    Ok(())
}

#[test]
fn optimised_query_4() -> Result<(), JasonError> {
    let source = InMemory::new();
    let mut database = composers_db(source)?.with_index(field!(year_of_birth))?;

    // Get only 19th-century composers
    let query = query!(year_of_birth >= 1800) & query!(name == "Johannes Brahms");

    let composers: Vec<String> = query
        .execute_optimised(&mut database)?
        .flatten()
        .map(|(_, person)| person.name)
        .collect();

    assert_eq!(composers.len(), 1);
    assert!(composers.contains(&"Johannes Brahms".to_string()));

    Ok(())
}

#[test]
fn unoptimised_query() -> Result<(), JasonError> {
    let source = InMemory::new();
    let mut database = composers_db(source)?;

    // Get only 19th-century composers
    let query = query!(year_of_birth >= 1800) & query!(year_of_birth < 1900);

    let composers: Vec<String> = query
        .execute_unoptimised(&mut database)?
        .flatten()
        .map(|(_, person)| person.name)
        .collect();

    assert_eq!(composers.len(), 3);
    assert!(composers.contains(&"Johannes Brahms".to_string()));
    assert!(composers.contains(&"Camille Saint-Saëns".to_string()));
    assert!(composers.contains(&"Pyotr Ilyich Tchaikovsky".to_string()));

    Ok(())
}

#[test]
fn into_file() -> Result<(), JasonError> {
    let source = InMemory::new();
    let database = composers_db(source)?;

    let mut file_database = database.into_file("test_into_file.jdb")?;
    let contents = file_database
        .iter()
        .flatten()
        .map(|(_, v)| v)
        .collect::<Vec<_>>();

    let person_1 = Person::new("Johann Sebastian Bach", 1685);
    let person_2 = Person::new("Wolfgang Amadeus Mozart", 1756);
    let person_3 = Person::new("Johannes Brahms", 1833);
    let person_4 = Person::new("Camille Saint-Saëns", 1835);
    let person_5 = Person::new("Pyotr Ilyich Tchaikovsky", 1840);
    let person_6 = Person::new("Dmitri Shostakovich", 1906);

    assert_eq!(contents.len(), 6);

    assert!(contents.contains(&person_1));
    assert!(contents.contains(&person_2));
    assert!(contents.contains(&person_3));
    assert!(contents.contains(&person_4));
    assert!(contents.contains(&person_5));
    assert!(contents.contains(&person_6));

    fs::remove_file("test_into_file.jdb").unwrap();

    Ok(())
}

#[test]
fn migration() -> Result<(), JasonError> {
    let source = InMemory::new();
    let database = composers_db(source)?;

    // Replace birth years with ages in 2022
    let mut database =
        database.migrate(|person| AgedPerson::new(person.name, 2022 - person.year_of_birth))?;

    assert_eq!(database.iter().count(), 6);

    assert_eq!(
        database.get("bach")?,
        AgedPerson::new("Johann Sebastian Bach", 337)
    );

    assert_eq!(
        database.get("mozart")?,
        AgedPerson::new("Wolfgang Amadeus Mozart", 266)
    );

    assert_eq!(
        database.get("brahms")?,
        AgedPerson::new("Johannes Brahms", 189)
    );

    assert_eq!(
        database.get("saint_saens")?,
        AgedPerson::new("Camille Saint-Saëns", 187)
    );

    assert_eq!(
        database.get("tchaikovsky")?,
        AgedPerson::new("Pyotr Ilyich Tchaikovsky", 182)
    );

    assert_eq!(
        database.get("shostakovich")?,
        AgedPerson::new("Dmitri Shostakovich", 116)
    );

    Ok(())
}
