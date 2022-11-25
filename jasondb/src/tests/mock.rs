use crate::error::JasonError;
use crate::sources::Source;
use crate::Database;

use humphrey_json::prelude::*;

#[derive(FromJson, IntoJson, Clone, Debug, PartialEq, Eq)]
pub struct Person {
    pub(crate) name: String,
    pub(crate) year_of_birth: u16,
}

#[derive(FromJson, IntoJson, Debug, PartialEq, Eq)]
pub struct AgedPerson {
    pub(crate) name: String,
    pub(crate) age: u16,
}

impl Person {
    pub fn new(name: impl AsRef<str>, year_of_birth: u16) -> Person {
        Person {
            name: name.as_ref().to_string(),
            year_of_birth,
        }
    }
}

impl AgedPerson {
    pub fn new(name: impl AsRef<str>, age: u16) -> AgedPerson {
        AgedPerson {
            name: name.as_ref().to_string(),
            age,
        }
    }
}

pub fn composers_db<S>(source: S) -> Result<Database<Person, S>, JasonError>
where
    S: Source,
{
    let mut database: Database<Person, S> = Database::from_source(source)?;

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

    Ok(database)
}
