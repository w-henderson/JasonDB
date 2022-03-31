use humphrey_json::prelude::*;

#[derive(Debug, PartialEq, Eq)]
pub struct Person {
    name: String,
    year_of_birth: u16,
}

impl Person {
    pub fn new(name: impl AsRef<str>, year_of_birth: u16) -> Person {
        Person {
            name: name.as_ref().to_string(),
            year_of_birth,
        }
    }
}

json_map! {
    Person,
    name => "name",
    year_of_birth => "yearOfBirth"
}
