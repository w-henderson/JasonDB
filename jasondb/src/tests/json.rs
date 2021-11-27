#[allow(unused_imports)]
use crate::database::Document;

#[test]
fn test_valid_json() {
    let valid_json = [
        r#"{"name": "William Henderson"}"#,
        r#""William Henderson""#,
        r#"{"age": 16}"#,
        r#"{"height": {"feet": 6, "inches": 0}}"#,
        r#"1337"#,
        r#"{"middleNames": ["Edward", "Haswell"]}"#,
        r#"{"emptyArray": []}"#,
    ];

    for json in &valid_json {
        assert!(Document::new("test".to_string(), json.to_string()).is_some());
    }
}

#[test]
#[cfg(feature = "validation")]
fn test_invalid_json() {
    let invalid_json = [
        r#"{"name": "William Henderson}"#,
        r#"William Henderson"#,
        r#"{"age": 16..}"#,
        r#"{"height": {"feet": six, "inches": 0}}"#,
        r#"1337a"#,
        r#"{"middleNames":]}"#,
    ];

    for json in &invalid_json {
        assert!(Document::new("test".to_string(), json.to_string()).is_none());
    }
}
