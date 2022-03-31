# JasonDB
JasonDB is a NoSQL, document-oriented, JSON-based database management system built with the modern web in mind. It is fast, flexible, and easy-to-use, making it a solid choice for building databases for web applications. It also provides a number of macros allowing for powerful operations in concise syntax.

## Installation
The JasonDB crate can be installed by adding `jasondb` to your `Cargo.toml` file.

## Documentation
The JasonDB documentation can be found at [docs.rs](https://docs.rs/jasondb).

## Basic Example
```rs
use jasondb::JasonDB;
use jasondb::prelude::*;

fn main() {
    // Create a new database (use `JasonDB::open` to open an existing database)
    let db = JasonDB::new("/path/to/database.jdb");

    // Lock the database for writing, then write to it
    // We do this in a new scope so the database is unlocked as soon as we're done
    {
        let mut db_write = db.write();
        set!(&mut db_write, "users/w-henderson", "{\"name\": \"William Henderson\"}");
        set!(&mut db_write, "users/torvalds", "{\"name\": \"Linus Torvalds\"}");
    }

    // Lock the database for reading, then read from it
    // Note that this is a contrived example (one could read from the write-locked database above)
    {
        let db_read = db.read();
        let test = get!(&db_read, "users/w-henderson");
        assert_eq!(test.json, "{\"name\": \"William Henderson\"}");
    }
}
```

## Further Examples
- [Message Board Example](https://github.com/w-henderson/Humphrey/tree/legacy/examples/database): A simple example of integrating JasonDB with a web application.