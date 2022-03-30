use crate::sources::Source;

use std::collections::HashMap;
use std::marker::PhantomData;

pub struct Database<T, S>
where
    S: Source,
{
    pub(crate) indexes: HashMap<String, u64>,
    pub(crate) file: S,
    marker: PhantomData<T>,
}
