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
        match self.predicate_combination {
            PredicateCombination::And => {
                // Check to see if a secondary index can be used to optimise this query
                let mut optimised_index = None;
                for predicate in &self.predicates {
                    let predicate_key = predicate.key();
                    if database.secondary_indexes.contains_key(predicate_key) {
                        optimised_index = Some(predicate_key);
                        break;
                    }
                }

                // If an optimisation was found, use it
                if let Some(optimised_index) = optimised_index {
                    let mut indexes: Vec<u64> = Vec::new();
                    let optimised_index = database.secondary_indexes.get(optimised_index).unwrap();

                    if self.predicates.len() == 1 {
                        if let Predicate::Eq(_, value) = &self.predicates[0] {
                            indexes = optimised_index.get(value).unwrap_or(&Vec::new()).to_vec();
                            return Ok(Iter {
                                database,
                                keys: indexes.into_iter(),
                            });
                        }
                    }

                    for (k, v) in optimised_index {
                        if self.matches(k)? {
                            indexes.extend(v.iter());
                        }
                    }

                    return Ok(Iter {
                        database,
                        keys: indexes.into_iter(),
                    });
                }
            }

            PredicateCombination::Or => {
                // TODO: implement this optimisation
            }
        }

        // No optimisation available so linear search is the only option.
        let mut indexes = Vec::new();
        let possible_indexes = database
            .primary_indexes
            .values()
            .cloned()
            .collect::<Vec<_>>();

        for index in possible_indexes {
            let value = database.get_at_index(index)?.1.to_json();

            if self.matches(&value)? {
                indexes.push(index);
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
