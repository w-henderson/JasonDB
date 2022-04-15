use crate::error::JasonError;
use crate::replica::Replica;
use crate::sources::InMemory;
use crate::Database;

use crate::tests::mock::Person;

use std::fs;
use std::sync::mpsc::{channel, Sender};

#[test]
fn sync_replica() -> Result<(), JasonError> {
    let mut database: Database<Person, InMemory> = Database::new_in_memory()
        .with_index("yearOfBirth")?
        .with_replica(Database::create("test_sync_replica.jdb")?);

    let person_1 = Person::new("Elizabeth II", 1926);
    let person_2 = Person::new("George VI", 1895);
    let person_3 = Person::new("Edward VIII", 1894);

    database.set("queen_elizabeth_ii", &person_1)?;
    database.set("king_george_vi", &person_2)?;
    database.set("king_edward_viii", &person_3)?;

    assert_eq!(database.iter().count(), 3);
    assert_eq!(database.get("queen_elizabeth_ii"), Ok(person_1.clone()));
    assert_eq!(database.get("king_george_vi"), Ok(person_2.clone()));
    assert_eq!(database.get("king_edward_viii"), Ok(person_3.clone()));

    drop(database);

    let mut database: Database<Person> = Database::open("test_sync_replica.jdb")?;

    assert_eq!(database.iter().count(), 3);
    assert_eq!(database.get("queen_elizabeth_ii"), Ok(person_1));
    assert_eq!(database.get("king_george_vi"), Ok(person_2));
    assert_eq!(database.get("king_edward_viii"), Ok(person_3));

    fs::remove_file("test_sync_replica.jdb").unwrap();

    Ok(())
}

#[test]
fn async_replica() -> Result<(), JasonError> {
    let mut database: Database<Person, InMemory> = Database::new_in_memory()
        .with_index("yearOfBirth")?
        .with_async_replica(Database::create("test_async_replica.jdb")?);

    let person_1 = Person::new("Elizabeth II", 1926);
    let person_2 = Person::new("George VI", 1895);
    let person_3 = Person::new("Edward VIII", 1894);

    database.set("queen_elizabeth_ii", &person_1)?;
    database.set("king_george_vi", &person_2)?;
    database.set("king_edward_viii", &person_3)?;

    assert_eq!(database.iter().count(), 3);
    assert_eq!(database.get("queen_elizabeth_ii"), Ok(person_1.clone()));
    assert_eq!(database.get("king_george_vi"), Ok(person_2.clone()));
    assert_eq!(database.get("king_edward_viii"), Ok(person_3.clone()));

    drop(database);

    let mut database: Database<Person> = Database::open("test_async_replica.jdb")?;

    assert_eq!(database.iter().count(), 3);
    assert_eq!(database.get("queen_elizabeth_ii"), Ok(person_1));
    assert_eq!(database.get("king_george_vi"), Ok(person_2));
    assert_eq!(database.get("king_edward_viii"), Ok(person_3));

    fs::remove_file("test_async_replica.jdb").unwrap();

    Ok(())
}

#[test]
fn arbitrary_replica() -> Result<(), JasonError> {
    struct ChannelReplica(Sender<(String, String)>);

    impl<T> Replica<T> for ChannelReplica
    where
        T: Send + 'static,
    {
        fn set(&mut self, key: &str, value: &str) -> Result<(), JasonError> {
            self.0
                .send((key.to_string(), value.to_string()))
                .map_err(|_| JasonError::Io)
        }
    }

    let (tx_1, rx_1) = channel();
    let (tx_2, rx_2) = channel();

    let mut database: Database<Person, InMemory> = Database::new_in_memory()
        .with_index("yearOfBirth")?
        .with_replica(ChannelReplica(tx_1.clone()))
        .with_async_replica(ChannelReplica(tx_2.clone()));

    let person_1 = Person::new("Elizabeth II", 1926);
    let person_2 = Person::new("George VI", 1895);
    let person_3 = Person::new("Edward VIII", 1894);

    database.set("queen_elizabeth_ii", &person_1)?;
    database.set("king_george_vi", &person_2)?;
    database.set("king_edward_viii", &person_3)?;

    assert_eq!(database.iter().count(), 3);
    assert_eq!(database.get("queen_elizabeth_ii"), Ok(person_1.clone()));
    assert_eq!(database.get("king_george_vi"), Ok(person_2.clone()));
    assert_eq!(database.get("king_edward_viii"), Ok(person_3.clone()));

    drop(database);

    for rx in [rx_1, rx_2] {
        assert_eq!(
            rx.try_recv(),
            Ok((
                "queen_elizabeth_ii".to_string(),
                humphrey_json::to_string(&person_1)
            ))
        );

        assert_eq!(
            rx.try_recv(),
            Ok((
                "king_george_vi".to_string(),
                humphrey_json::to_string(&person_2)
            ))
        );

        assert_eq!(
            rx.try_recv(),
            Ok((
                "king_edward_viii".to_string(),
                humphrey_json::to_string(&person_3)
            ))
        );

        assert!(rx.try_recv().is_err());
    }

    drop(tx_1);
    drop(tx_2);

    Ok(())
}
