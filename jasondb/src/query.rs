use std::ops::{BitAnd, BitOr};

pub use humphrey_json::Value;

use crate::error::JasonError;

#[derive(Debug, PartialEq)]
pub struct Query {
    pub(crate) predicates: Vec<Predicate>,
    pub(crate) predicate_combination: PredicateCombination,
}

#[derive(Debug, PartialEq)]
pub enum Predicate {
    Gt(String, f64),
    Gte(String, f64),
    Lt(String, f64),
    Lte(String, f64),
    Eq(String, Value),
}

#[derive(Debug, PartialEq)]
pub enum PredicateCombination {
    And,
    Or,
}

impl Query {
    pub fn matches(&self, json: &Value) -> Result<bool, JasonError> {
        match self.predicate_combination {
            PredicateCombination::And => {
                for predicate in &self.predicates {
                    if !predicate.matches(json)? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            PredicateCombination::Or => {
                for predicate in &self.predicates {
                    if predicate.matches(json)? {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
        }
    }
}

impl Predicate {
    pub fn matches(&self, json: &Value) -> Result<bool, JasonError> {
        match self {
            Self::Gt(index, right) => {
                let left = Self::get_number(index, json)?;
                Ok(left > *right)
            }
            Self::Gte(index, right) => {
                let left = Self::get_number(index, json)?;
                Ok(left >= *right)
            }
            Self::Lt(index, right) => {
                let left = Self::get_number(index, json)?;
                Ok(left < *right)
            }
            Self::Lte(index, right) => {
                let left = Self::get_number(index, json)?;
                Ok(left <= *right)
            }
            Self::Eq(index, right) => {
                let left = Self::get_value(index, json)?;
                Ok(left == *right)
            }
        }
    }

    fn get_value(index: &str, json: &Value) -> Result<Value, JasonError> {
        let indexing_path = index.split('.');
        let mut current_json = json;
        for index in indexing_path {
            current_json = current_json.get(index).ok_or(JasonError::JsonError)?;
        }

        Ok(current_json.clone())
    }

    fn get_number(index: &str, json: &Value) -> Result<f64, JasonError> {
        let value = Self::get_value(index, json)?;
        let number = value.as_number().ok_or(JasonError::JsonError)?;

        Ok(number)
    }
}

impl From<Predicate> for Query {
    fn from(predicate: Predicate) -> Self {
        Self {
            predicates: vec![predicate],
            predicate_combination: PredicateCombination::And,
        }
    }
}

impl BitAnd for Query {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self {
        Self {
            predicates: self.predicates.into_iter().chain(rhs.predicates).collect(),
            predicate_combination: PredicateCombination::And,
        }
    }
}

impl BitOr for Query {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self {
        Self {
            predicates: self.predicates.into_iter().chain(rhs.predicates).collect(),
            predicate_combination: PredicateCombination::Or,
        }
    }
}

#[macro_export]
macro_rules! query {
    ($($field:ident).+ > $value:expr) => {
        $crate::query::Query::from($crate::query::Predicate::Gt(
            stringify!($($field).+).to_string(),
            f64::from($value),
        ))
    };

    ($($field:ident).+ >= $value:expr) => {
        $crate::query::Query::from($crate::query::Predicate::Gte(
            stringify!($($field).+).to_string(),
            f64::from($value),
        ))
    };

    ($($field:ident).+ < $value:expr) => {
        $crate::query::Query::from($crate::query::Predicate::Lt(
            stringify!($($field).+).to_string(),
            f64::from($value),
        ))
    };

    ($($field:ident).+ <= $value:expr) => {
        $crate::query::Query::from($crate::query::Predicate::Lte(
            stringify!($($field).+).to_string(),
            f64::from($value),
        ))
    };

    ($($field:ident).+ == null) => {
        $crate::query::Query::from($crate::query::Predicate::Eq(
            stringify!($($field).+).to_string(),
            $crate::query::Value::Null,
        ))
    };

    ($($field:ident).+ == $value:expr) => {
        $crate::query::Query::from($crate::query::Predicate::Eq(
            stringify!($($field).+).to_string(),
            $crate::query::Value::from($value),
        ))
    };

    ($($field:ident).+) => {
        $crate::query::Query::from($crate::query::Predicate::Eq(
            stringify!($($field).+).to_string(),
            $crate::query::Value::Bool(true),
        ))
    };
}
