use crate::database::{Database, Iter};
use crate::error::JasonError;
use crate::sources::Source;
use crate::util::indexing;

use humphrey_json::prelude::*;
pub use humphrey_json::Value;

use std::ops::{BitAnd, BitOr};

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
    pub fn execute<'a, T, S>(
        &self,
        database: &'a mut Database<T, S>,
    ) -> Result<Iter<'a, T, S>, JasonError>
    where
        T: IntoJson + FromJson,
        S: Source,
    {
        let first_predicate_key = self.predicates[0].key();

        let optimisable = database.secondary_indexes.contains_key(first_predicate_key)
            && self
                .predicates
                .iter()
                .map(|p| p.key())
                .all(|k| k == first_predicate_key);

        if optimisable {
            self.execute_optimised(database)
        } else {
            self.execute_unoptimised(database)
        }
    }

    pub fn execute_optimised<'a, T, S>(
        &self,
        database: &'a mut Database<T, S>,
    ) -> Result<Iter<'a, T, S>, JasonError>
    where
        T: IntoJson + FromJson,
        S: Source,
    {
        let mut indexes = Vec::new();

        let secondary_index = database
            .secondary_indexes
            .get(self.predicates[0].key())
            .ok_or(JasonError::JsonError)?;

        for (value, value_indexes) in secondary_index {
            if self.matches_direct(value)? {
                indexes.extend(value_indexes);
            }
        }

        Ok(Iter {
            database,
            keys: indexes.into_iter(),
        })
    }

    pub fn execute_unoptimised<'a, T, S>(
        &self,
        database: &'a mut Database<T, S>,
    ) -> Result<Iter<'a, T, S>, JasonError>
    where
        T: IntoJson + FromJson,
        S: Source,
    {
        let mut indexes = Vec::new();
        let keys = database
            .primary_indexes
            .values()
            .cloned()
            .collect::<Vec<_>>();

        for key in &keys {
            let (_, v) = database.get_at_index(*key)?;

            if self.matches(&v.to_json())? {
                indexes.push(key.to_owned());
            }
        }

        Ok(Iter {
            database,
            keys: indexes.into_iter(),
        })
    }

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

    pub fn matches_direct(&self, json: &Value) -> Result<bool, JasonError> {
        match self.predicate_combination {
            PredicateCombination::And => {
                for predicate in &self.predicates {
                    if !predicate.matches_direct(json)? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            PredicateCombination::Or => {
                for predicate in &self.predicates {
                    if predicate.matches_direct(json)? {
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
                let left = indexing::get_number(index, json)?;
                Ok(left > *right)
            }
            Self::Gte(index, right) => {
                let left = indexing::get_number(index, json)?;
                Ok(left >= *right)
            }
            Self::Lt(index, right) => {
                let left = indexing::get_number(index, json)?;
                Ok(left < *right)
            }
            Self::Lte(index, right) => {
                let left = indexing::get_number(index, json)?;
                Ok(left <= *right)
            }
            Self::Eq(index, right) => {
                let left = indexing::get_value(index, json)?;
                Ok(left == *right)
            }
        }
    }

    pub fn matches_direct(&self, json: &Value) -> Result<bool, JasonError> {
        match self {
            Self::Gt(_, right) => {
                let left = json.as_number().ok_or(JasonError::JsonError)?;
                Ok(left > *right)
            }
            Self::Gte(_, right) => {
                let left = json.as_number().ok_or(JasonError::JsonError)?;
                Ok(left >= *right)
            }
            Self::Lt(_, right) => {
                let left = json.as_number().ok_or(JasonError::JsonError)?;
                Ok(left < *right)
            }
            Self::Lte(_, right) => {
                let left = json.as_number().ok_or(JasonError::JsonError)?;
                Ok(left <= *right)
            }
            Self::Eq(_, right) => Ok(*json == *right),
        }
    }

    pub fn key(&self) -> &str {
        match self {
            Self::Gt(key, _) => key,
            Self::Gte(key, _) => key,
            Self::Lt(key, _) => key,
            Self::Lte(key, _) => key,
            Self::Eq(key, _) => key,
        }
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

#[macro_export]
macro_rules! field {
    ($($field:ident).+) => {
        stringify!($($field).+).to_string()
    }
}
