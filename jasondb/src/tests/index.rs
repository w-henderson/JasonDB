use crate::error::JasonError;
use crate::sources::InMemory;
use crate::Database;

use crate::tests::mock::Person;

use humphrey_json::Value;

use std::collections::{BTreeSet, HashMap};

#[test]
fn test_add_new() -> Result<(), JasonError> {
    let mut database: Database<Person, InMemory> = Database::new_in_memory()
        .with_index("name")?
        .with_index("year_of_birth")?;

    let person_1 = Person::new("A", 2000);
    let person_2 = Person::new("B", 2000);
    let person_3 = Person::new("C", 2001);
    let person_4 = Person::new("D", 2002);

    database.set("a", &person_1)?;
    database.set("b", &person_2)?;
    database.set("c", &person_3)?;
    database.set("d", &person_4)?;

    let index_1 = *database.primary_indexes.get("a").unwrap();
    let index_2 = *database.primary_indexes.get("b").unwrap();
    let index_3 = *database.primary_indexes.get("c").unwrap();
    let index_4 = *database.primary_indexes.get("d").unwrap();

    let name_index = database.secondary_indexes.get("name").unwrap();
    let year_of_birth_index = database.secondary_indexes.get("year_of_birth").unwrap();

    let expected_name_index: HashMap<Value, BTreeSet<u64>> = [
        (
            Value::String("A".to_string()),
            [index_1].iter().cloned().collect(),
        ),
        (
            Value::String("B".to_string()),
            [index_2].iter().cloned().collect(),
        ),
        (
            Value::String("C".to_string()),
            [index_3].iter().cloned().collect(),
        ),
        (
            Value::String("D".to_string()),
            [index_4].iter().cloned().collect(),
        ),
    ]
    .into();

    let expected_year_of_birth_index: HashMap<Value, BTreeSet<u64>> = [
        (
            Value::Number(2000.0),
            [index_1, index_2].iter().cloned().collect(),
        ),
        (Value::Number(2001.0), [index_3].iter().cloned().collect()),
        (Value::Number(2002.0), [index_4].iter().cloned().collect()),
    ]
    .into();

    assert_eq!(*name_index, expected_name_index);
    assert_eq!(*year_of_birth_index, expected_year_of_birth_index);

    Ok(())
}

#[test]
fn test_update() -> Result<(), JasonError> {
    let mut database: Database<Person, InMemory> = Database::new_in_memory()
        .with_index("name")?
        .with_index("year_of_birth")?;

    let person_1 = Person::new("A", 2000);
    let person_2 = Person::new("B", 2000);
    let person_3 = Person::new("C", 2001);
    let person_4 = Person::new("D", 2002);

    database.set("a", &person_1)?;
    database.set("b", &person_2)?;
    database.set("c", &person_3)?;
    database.set("d", &person_4)?;

    let updated_person_1 = Person::new("A", 2001);
    database.set("a", &updated_person_1)?;

    let index_1 = *database.primary_indexes.get("a").unwrap();
    let index_2 = *database.primary_indexes.get("b").unwrap();
    let index_3 = *database.primary_indexes.get("c").unwrap();
    let index_4 = *database.primary_indexes.get("d").unwrap();

    let name_index = database.secondary_indexes.get("name").unwrap();
    let year_of_birth_index = database.secondary_indexes.get("year_of_birth").unwrap();

    let expected_name_index: HashMap<Value, BTreeSet<u64>> = [
        (
            Value::String("A".to_string()),
            [index_1].iter().cloned().collect(),
        ),
        (
            Value::String("B".to_string()),
            [index_2].iter().cloned().collect(),
        ),
        (
            Value::String("C".to_string()),
            [index_3].iter().cloned().collect(),
        ),
        (
            Value::String("D".to_string()),
            [index_4].iter().cloned().collect(),
        ),
    ]
    .into();

    let expected_year_of_birth_index: HashMap<Value, BTreeSet<u64>> = [
        (Value::Number(2000.0), [index_2].iter().cloned().collect()),
        (
            Value::Number(2001.0),
            [index_3, index_1].iter().cloned().collect(),
        ),
        (Value::Number(2002.0), [index_4].iter().cloned().collect()),
    ]
    .into();

    assert_eq!(*name_index, expected_name_index);
    assert_eq!(*year_of_birth_index, expected_year_of_birth_index);

    Ok(())
}
