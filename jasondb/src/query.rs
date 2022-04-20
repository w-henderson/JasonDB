//! Provides query construction functionality.

use crate::database::{Database, Iter};
use crate::error::JasonError;
use crate::sources::Source;
use crate::util::indexing;

use humphrey_json::prelude::*;
pub use humphrey_json::Value;

use std::fmt::Debug;
use std::ops::{BitAnd, BitOr};

/// Represents a query to be executed against a database.
///
/// Created with the `query!` macro.
#[derive(Debug, PartialEq)]
pub struct Query {
    pub(crate) predicates: Vec<Predicate>,
    pub(crate) predicate_combination: PredicateCombination,
}

/// Represents a predicate as part of a query.
///
/// Created with the `query!` macro.
#[derive(Debug, PartialEq)]
pub enum Predicate {
    /// Equivalent to `key > value`.
    Gt(String, f64),
    /// Equivalent to `key >= value`.
    Gte(String, f64),
    /// Equivalent to `key < value`.
    Lt(String, f64),
    /// Equivalent to `key <= value`.
    Lte(String, f64),
    /// Equivalent to `key == value`.
    Eq(String, Value),
    /// Equivalent to `key != value`.
    Ne(String, Value),
    /// Equivalent to `closure(key)`.
    Closure(String, PredicateClosure),
}

/// Represents a way of combining predicates. Currently the options are `and` and `or`.
#[derive(Debug, PartialEq)]
pub enum PredicateCombination {
    /// Equivalent to logical `&&`.
    And,
    /// Equivalent to logical `||`.
    Or,
}

/// Represents a closure that can be used as a predicate.
pub struct PredicateClosure {
    /// The closure which checks whether the predicate matches the value.
    pub closure: Box<dyn Fn(&Value) -> bool>,
}

impl Query {
    /// Attempts to execute the query against the given database.
    ///
    /// If successful, an iterator over the matching values is returned.
    /// This will automatically optimise the query where possible
    ///   (see issue [#9](https://github.com/w-henderson/JasonDB/issues/9) for optimisation status).
    pub fn execute<'a, T, S>(
        &self,
        database: &'a mut Database<T, S>,
    ) -> Result<Iter<'a, T, S>, JasonError>
    where
        T: IntoJson + FromJson,
        S: Source,
    {
        if self.is_optimisable(database) {
            self.execute_optimised(database)
        } else {
            self.execute_unoptimised(database)
        }
    }

    /// Checks whether the query is optimisable on the given database.
    ///
    /// This is used to prevent unnecessary optimisation attempts on unoptimisable queries.
    fn is_optimisable<T, S>(&self, database: &Database<T, S>) -> bool
    where
        T: IntoJson + FromJson,
        S: Source,
    {
        match self.predicate_combination {
            PredicateCombination::And => self.predicates.iter().any(|p| p.is_indexed(database)),
            PredicateCombination::Or => self.predicates.iter().all(|p| p.is_indexed(database)),
        }
    }

    /// Executes the query.
    pub(crate) fn execute_optimised<'a, T, S>(
        &self,
        database: &'a mut Database<T, S>,
    ) -> Result<Iter<'a, T, S>, JasonError>
    where
        T: IntoJson + FromJson,
        S: Source,
    {
        let mut indexes = Vec::new();

        let optimisable_predicates = self
            .predicates
            .iter()
            .filter(|p| database.secondary_indexes.contains_key(p.key()))
            .collect::<Vec<_>>();

        let unoptimisable_predicates = self
            .predicates
            .iter()
            .filter(|p| !database.secondary_indexes.contains_key(p.key()))
            .collect::<Vec<_>>();

        // Use each predicate's corresponding index to find matches.
        for predicate in &optimisable_predicates {
            let index = database.secondary_indexes.get(predicate.key()).unwrap();

            for (v, i) in index {
                if predicate.matches_direct(v)? {
                    indexes.extend(i.iter());
                }
            }
        }

        let include: Box<dyn Fn(usize) -> bool> = match self.predicate_combination {
            PredicateCombination::And => Box::new(|n: usize| n == optimisable_predicates.len()),
            PredicateCombination::Or => Box::new(|n: usize| n > 0),
        };

        let mut combined_indexes = Vec::new();
        let mut count = 0;
        let mut last = 1; // cannot be a real index so we're good

        // We don't want an unstable sort because the regular one is quicker.
        // This is because the concatenated indexes are already sorted so it's just sorting a list of sorted lists.
        // (yes, this has been verified by benchmarks, it's ~2.5x faster)
        #[allow(clippy::stable_sort_primitive)]
        indexes.sort();

        // Use the number of matches found to determine which indexes meet the predicate combination requirements.
        // If the number of matches is equal to the number of predicates, then the `And` combination is met.
        // If the number of matches is greater than 0, then the `Or` combination is met.
        // Otherwise, neither is met.
        for index in indexes {
            if last != index {
                if include(count) {
                    combined_indexes.push(last);
                }

                last = index;
                count = 1;
            } else {
                count += 1;
            }
        }

        if include(count) {
            combined_indexes.push(last);
        }

        if unoptimisable_predicates.is_empty() {
            // If there are no unoptimisable predicates, we don't need to check any more conditions and we can return now.

            Ok(Iter {
                database,
                keys: combined_indexes.into_iter(),
            })
        } else {
            // If there are some unoptimisable predicates, we check them manually but use the existing indexes instead of every index.
            // This is quicker than iterating over the whole database, but can only be applied when the combination is `And`.

            let mut filtered_indexes = Vec::with_capacity(combined_indexes.len());

            'outer: for index in combined_indexes {
                let (_, v) = database.get_at_index(index)?;

                for predicate in &unoptimisable_predicates {
                    if !predicate.matches(&v.to_json())? {
                        continue 'outer;
                    }
                }

                filtered_indexes.push(index);
            }

            Ok(Iter {
                database,
                keys: filtered_indexes.into_iter(),
            })
        }
    }

    /// Executes the query with no optimisations.
    pub(crate) fn execute_unoptimised<'a, T, S>(
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
                indexes.push(*key);
            }
        }

        Ok(Iter {
            database,
            keys: indexes.into_iter(),
        })
    }

    /// Checks whether the query matches the given value.
    pub(crate) fn matches(&self, json: &Value) -> Result<bool, JasonError> {
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
    /// Checks whether the predicate is indexed by the given database.
    fn is_indexed<T, S>(&self, database: &Database<T, S>) -> bool
    where
        T: IntoJson + FromJson,
        S: Source,
    {
        database.secondary_indexes.contains_key(self.key())
    }

    /// Checks whether the predicate matches the given value.
    pub(crate) fn matches(&self, json: &Value) -> Result<bool, JasonError> {
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
                let left = indexing::get_value(index, json);
                Ok(left == *right)
            }
            Self::Ne(index, right) => {
                let left = indexing::get_value(index, json);
                Ok(left != *right)
            }
            Self::Closure(index, closure) => {
                let left = indexing::get_value(index, json);
                Ok((closure.closure)(&left))
            }
        }
    }

    /// Checks whether the predicate directly matches the given value.
    /// This bypasses the index and checks for equality with the value itself.
    pub(crate) fn matches_direct(&self, json: &Value) -> Result<bool, JasonError> {
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
            Self::Ne(_, right) => Ok(*json != *right),
            Self::Closure(_, closure) => Ok((closure.closure)(json)),
        }
    }

    /// Returns the key of the predicate.
    pub(crate) fn key(&self) -> &str {
        match self {
            Self::Gt(key, _) => key,
            Self::Gte(key, _) => key,
            Self::Lt(key, _) => key,
            Self::Lte(key, _) => key,
            Self::Eq(key, _) => key,
            Self::Ne(key, _) => key,
            Self::Closure(key, _) => key,
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

impl Debug for PredicateClosure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PredicateClosure").finish()
    }
}

impl PartialEq for PredicateClosure {
    fn eq(&self, _: &Self) -> bool {
        // Closures cannot be equal
        false
    }
}

/// Creates a query from Rust-like logical syntax.
///
/// ## Basic Examples
/// For simple equality and ordering queries, you can just write the path to the field as you would in Rust or JavaScript,
///   separated by dots, then an operator, and then a value. This uses the field names in the JSON serialization instead
///   of those in the Rust struct.
///
/// ```
/// query!(age >= 18) // `age` field >= 18
/// query!(coordinates.lat > 0.0) // `lat` field of `coordinates` > 0.0, e.g. above equator
/// query!(country == "UK") // `country` field == "UK"
/// query!(price < 10) | query!(discounted) // `price` field < 10 or `discounted` field == true
/// ```
///
/// You'll notice that queries are combined using bitwise operators outside of the macro.
/// This is because the macro is currently not able to parse `&&` and `||`, but this will hopefully change in the future.
///
/// ## Advanced Examples
/// For more complex queries, you can use a closure to define the predicate. You still need to specify the field using the dot
///   syntax for optimisation purposes, as shown below.
///
/// ```
/// // Check whether the field `dob.year` is a leap year.
/// // https://en.wikipedia.org/wiki/Leap_year
/// query!(dob.year, |year| year
///     .as_number()
///     .map(|y| (y as usize % 4 == 0 && y as usize % 100 != 0) || y as usize % 400 == 0)
///     .unwrap_or(false));
/// ```
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

    ($($field:ident).+ != null) => {
        $crate::query::Query::from($crate::query::Predicate::Ne(
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

    ($($field:ident).+ != $value:expr) => {
        $crate::query::Query::from($crate::query::Predicate::Ne(
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

    ($($field:ident).+, $closure:expr) => {
        $crate::query::Query::from($crate::query::Predicate::Closure(
            stringify!($($field).+).to_string(),
            $crate::query::PredicateClosure {
                closure: Box::new($closure),
            },
        ))
    };
}

/// Creates a field string from Rust-like field access syntax.
///
/// ## Examples
/// ```
/// assert_eq!(field!(coordinates.lat), "coordinates.lat");
/// assert_eq!(field!(age), "age");
/// ```
#[macro_export]
macro_rules! field {
    ($($field:ident).+) => {
        stringify!($($field).+).to_string()
    }
}
