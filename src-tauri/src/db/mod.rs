pub mod assessment;
pub mod coaching_cache;
pub mod connection;
pub mod game;
pub mod player;
pub mod puzzle;
pub mod repertoire;

/// Extension trait to convert `QueryReturnedNoRows` into `Ok(None)`.
pub(crate) trait OptionalRow {
    type Output;
    fn optional(self) -> Result<Option<Self::Output>, rusqlite::Error>;
}

impl<T> OptionalRow for Result<T, rusqlite::Error> {
    type Output = T;
    fn optional(self) -> Result<Option<T>, rusqlite::Error> {
        match self {
            Ok(v) => Ok(Some(v)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod migration_tests;
