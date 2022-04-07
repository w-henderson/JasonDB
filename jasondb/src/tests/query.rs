use humphrey_json::prelude::*;

#[test]
fn test_queries() {
    let query_1 = query!(a < 1);
    let query_2 = query!(b.c >= 2);
    let query_3 = query!(c == false);
    let query_4 = query!(d == "hello");
    let query_5 = query!(d != "hello");
    let compound_query_1 = query!(a < 1) & query!(c);
    let compound_query_2 = query!(a < 1) | query!(c);

    let testcase_1 = json!({
        "a": 0,
        "b": {
            "c": 1
        },
        "c": true,
        "d": "goodbye"
    });

    let testcase_2 = json!({
        "a": 1,
        "b": {
            "c": 2
        },
        "c": true,
        "d": "hello"
    });

    let testcase_3 = json!({
        "a": 2,
        "b": {
            "c": 3
        },
        "c": false,
        "d": "goodbye"
    });

    assert!(query_1.matches(&testcase_1).unwrap());
    assert!(!query_1.matches(&testcase_2).unwrap());
    assert!(!query_1.matches(&testcase_3).unwrap());

    assert!(!query_2.matches(&testcase_1).unwrap());
    assert!(query_2.matches(&testcase_2).unwrap());
    assert!(query_2.matches(&testcase_3).unwrap());

    assert!(!query_3.matches(&testcase_1).unwrap());
    assert!(!query_3.matches(&testcase_2).unwrap());
    assert!(query_3.matches(&testcase_3).unwrap());

    assert!(!query_4.matches(&testcase_1).unwrap());
    assert!(query_4.matches(&testcase_2).unwrap());
    assert!(!query_4.matches(&testcase_3).unwrap());

    assert!(query_5.matches(&testcase_1).unwrap());
    assert!(!query_5.matches(&testcase_2).unwrap());
    assert!(query_5.matches(&testcase_3).unwrap());

    assert!(compound_query_1.matches(&testcase_1).unwrap());
    assert!(!compound_query_1.matches(&testcase_2).unwrap());
    assert!(!compound_query_1.matches(&testcase_3).unwrap());

    assert!(compound_query_2.matches(&testcase_1).unwrap());
    assert!(compound_query_2.matches(&testcase_2).unwrap());
    assert!(!compound_query_2.matches(&testcase_3).unwrap());
}
