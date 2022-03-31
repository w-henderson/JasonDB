use crate::query::{Predicate, PredicateCombination, Query};

use humphrey_json::Value;

#[test]
fn simple_queries() {
    let lt = query!(some.a < 1);
    let lte = query!(other.a.b <= 2);
    let gt = query!(some.a > 1.0);
    let gte = query!(other.a.b >= 2.0);
    let eq_num = query!(some.a == 1);
    let eq_str = query!(some.a == "hello");
    let eq_bool = query!(some.a == true);
    let eq_null = query!(some.a == null);
    let eq_var = query!(some.a == f64::MAX);

    assert_eq!(lt, Query::from(Predicate::Lt("some.a".to_string(), 1.0)));
    assert_eq!(
        lte,
        Query::from(Predicate::Lte("other.a.b".to_string(), 2.0))
    );
    assert_eq!(gt, Query::from(Predicate::Gt("some.a".to_string(), 1.0)));
    assert_eq!(
        gte,
        Query::from(Predicate::Gte("other.a.b".to_string(), 2.0))
    );
    assert_eq!(
        eq_num,
        Query::from(Predicate::Eq("some.a".to_string(), Value::Number(1.0)))
    );
    assert_eq!(
        eq_str,
        Query::from(Predicate::Eq(
            "some.a".to_string(),
            Value::String("hello".to_string())
        ))
    );
    assert_eq!(
        eq_bool,
        Query::from(Predicate::Eq("some.a".to_string(), Value::Bool(true)))
    );
    assert_eq!(
        eq_null,
        Query::from(Predicate::Eq("some.a".to_string(), Value::Null))
    );
    assert_eq!(
        eq_var,
        Query::from(Predicate::Eq("some.a".to_string(), Value::Number(f64::MAX)))
    );
}

#[test]
fn compound_queries() {
    let and = query!(some.a > 1) & query!(other.a.b < 2);
    let or = query!(some.a > 1) | query!(other.a.b < 2);

    assert_eq!(
        and,
        Query {
            predicates: vec![
                Predicate::Gt("some.a".to_string(), 1.0),
                Predicate::Lt("other.a.b".to_string(), 2.0),
            ],
            predicate_combination: PredicateCombination::And
        }
    );

    assert_eq!(
        or,
        Query {
            predicates: vec![
                Predicate::Gt("some.a".to_string(), 1.0),
                Predicate::Lt("other.a.b".to_string(), 2.0),
            ],
            predicate_combination: PredicateCombination::Or
        }
    );
}
