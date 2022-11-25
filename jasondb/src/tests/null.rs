use crate::error::JasonError;
use crate::sources::InMemory;
use crate::Database;

use humphrey_json::prelude::*;

#[derive(FromJson, IntoJson, Debug, PartialEq, Eq, Clone)]
struct NullableType {
    field: String,
    nullable_field: Option<String>,
    nested_nullable_type: Option<NestedNullableType>,
}

#[derive(FromJson, IntoJson, Debug, PartialEq, Eq, Clone)]
struct NestedNullableType {
    field: String,
    nullable_field: Option<String>,
}

#[test]
fn get_set_nullable() -> Result<(), Box<JasonError>> {
    let mut db: Database<NullableType, InMemory> = Database::new_in_memory();

    let value_1 = NullableType {
        field: "some value".to_string(),
        nullable_field: Some("some value".to_string()),
        nested_nullable_type: None,
    };

    let value_2 = NullableType {
        field: "none value".to_string(),
        nullable_field: None,
        nested_nullable_type: None,
    };

    db.set("key1", &value_1)?;
    db.set("key2", &value_2)?;

    assert_eq!(db.get("key1")?, value_1);
    assert_eq!(db.get("key2")?, value_2);
    assert!(db.get("key3").is_err());

    let mut db = db.with_index("field")?.with_index("nullable_field")?;

    assert_eq!(db.get("key1")?, value_1);
    assert_eq!(db.get("key2")?, value_2);
    assert!(db.get("key3").is_err());

    Ok(())
}

#[test]
fn get_set_nested_nullable() -> Result<(), Box<JasonError>> {
    let mut db: Database<NullableType, InMemory> = Database::new_in_memory();

    let value_1 = NullableType {
        field: "some value".to_string(),
        nullable_field: Some("some value".to_string()),
        nested_nullable_type: Some(NestedNullableType {
            field: "some value".to_string(),
            nullable_field: Some("some value".to_string()),
        }),
    };

    let value_2 = NullableType {
        field: "none value".to_string(),
        nullable_field: None,
        nested_nullable_type: Some(NestedNullableType {
            field: "none value".to_string(),
            nullable_field: None,
        }),
    };

    let value_3 = NullableType {
        field: "none value".to_string(),
        nullable_field: None,
        nested_nullable_type: None,
    };

    db.set("key1", &value_1)?;
    db.set("key2", &value_2)?;
    db.set("key3", &value_3)?;

    let mut db = db
        .with_index("field")?
        .with_index("nested_nullable_type.nullable_field")?;

    assert_eq!(db.get("key1")?, value_1);
    assert_eq!(db.get("key2")?, value_2);
    assert_eq!(db.get("key3")?, value_3);
    assert!(db.get("key4").is_err());

    Ok(())
}

#[test]
fn nested_nullable_query() -> Result<(), Box<JasonError>> {
    let mut db: Database<NullableType, InMemory> =
        Database::new_in_memory().with_index("nested_nullable_type.nullable_field")?;

    let value_1 = NullableType {
        field: "some value".to_string(),
        nullable_field: Some("some value".to_string()),
        nested_nullable_type: Some(NestedNullableType {
            field: "some value".to_string(),
            nullable_field: Some("some value".to_string()),
        }),
    };

    let value_2 = NullableType {
        field: "none value".to_string(),
        nullable_field: None,
        nested_nullable_type: Some(NestedNullableType {
            field: "none value".to_string(),
            nullable_field: None,
        }),
    };

    db.set("key1", &value_1)?;
    db.set("key2", &value_2)?;

    assert_eq!(
        db.query(query!(field == "none value"))
            .unwrap()
            .flatten()
            .map(|(_, item)| item)
            .collect::<Vec<NullableType>>(),
        vec![value_2.clone()]
    );

    Ok(())
}
