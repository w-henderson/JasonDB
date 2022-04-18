use crate::error::JasonError;
use crate::sources::InMemory;
use crate::tests::mock::composers_db;

#[test]
fn iter_ordered() -> Result<(), JasonError> {
    let mut db = composers_db(InMemory::new())?;
    let mut iter = db.iter().flatten().map(|(k, _)| k);

    assert_eq!(iter.next(), Some("bach".to_string()));
    assert_eq!(iter.next(), Some("mozart".to_string()));
    assert_eq!(iter.next(), Some("brahms".to_string()));
    assert_eq!(iter.next(), Some("saint_saens".to_string()));
    assert_eq!(iter.next(), Some("tchaikovsky".to_string()));
    assert_eq!(iter.next(), Some("shostakovich".to_string()));
    assert_eq!(iter.next(), None);

    Ok(())
}
