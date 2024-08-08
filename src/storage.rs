use crate::command::CommandError;
use crate::query::{Query, ResultSet};
use bincode::error::{DecodeError, EncodeError};
use serde::{Deserialize, Serialize};
use sled::Db;
use std::marker::PhantomData;
use std::path::Path;
use thiserror::Error;
use crate::query::reflect::Reflectable;

/// Persistent key-value storage.
pub struct Storage<V: Serialize + for<'a> Deserialize<'a>> {
    db: Db,
    phantom_data: PhantomData<V>,
}

impl<V: Serialize + for<'a> Deserialize<'a>> Storage<V> {
    /// Open storage with specified path.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, StorageError> {
        let db = sled::open(path)?;

        Ok(Self {
            phantom_data: PhantomData,
            db,
        })
    }
    /// Get value by key. Value will be deserialized by bincode.
    pub fn get<K: AsRef<[u8]>>(&self, key: K) -> Result<Option<V>, StorageError> {
        Ok(self
            .db
            .get(key)?
            .map(|data| {
                bincode::serde::decode_from_std_read(&mut &*data, bincode::config::standard())
            })
            .transpose()?)
    }
    /// Update value
    pub fn update<K: AsRef<[u8]>>(
        &self,
        key: K,
        update_fn: impl FnOnce(&mut V),
    ) -> Result<bool, StorageError> {
        let key = key.as_ref();
        let value = self.get(key)?;
        if let Some(mut value) = value {
            update_fn(&mut value);
            let updated_value = bincode::serde::encode_to_vec(value, bincode::config::standard())?;
            self.db.insert(key, updated_value)?;

            return Ok(true);
        }

        Ok(false)
    }
    /// Insert value. Value will be serialized by bincode.
    pub fn insert<K: AsRef<[u8]>>(&self, key: K, value: &V) -> Result<Option<V>, StorageError> {
        let value = bincode::serde::encode_to_vec(value, bincode::config::standard())?;
        let old_value = self.db.insert(key, value)?;

        Ok(old_value
            .map(|x| bincode::serde::decode_from_std_read(&mut &*x, bincode::config::standard()))
            .transpose()?)
    }

    pub fn delete<K: AsRef<[u8]>>(&self, key: K) -> Result<Option<V>, StorageError> {
        let old_value = self.db.remove(key)?;

        Ok(old_value
            .map(|x| bincode::serde::decode_from_std_read(&mut &*x, bincode::config::standard()))
            .transpose()?)
    }
}

impl<V: Reflectable + for<'a> Deserialize<'a> + Serialize> Storage<V> {
    /// Select values that satisfy query.
    pub fn select(&self, query: Query) -> Result<ResultSet, CommandError> {
        let items = self
            .db
            .iter()
            .values()
            .map(|x| {
                x.map_err(Into::into).and_then(|data| {
                    bincode::serde::decode_from_std_read(&mut &*data, bincode::config::standard())
                        .map_err(Into::into)
                })
            })
            .collect::<Result<Vec<V>, StorageError>>()?;

        Ok(query.execute(items.iter())?)
    }
}

/// Represents possible errors of running command.
#[derive(Error, Debug)]
pub enum StorageError {
    #[error(transparent)]
    Sled(#[from] sled::Error),
    #[error(transparent)]
    Encode(#[from] EncodeError),
    #[error(transparent)]
    Decode(#[from] DecodeError),
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use tempfile::tempdir;
    use crate::query::evaluator::query::tests::test_dataset;
    use crate::query::reflect::Value;
    use super::*;

    #[test]
    fn get_item() {
        let storage = get_test_storage();
        let test_dataset = test_dataset();

        for test in &test_dataset{
            storage.insert(&test.string, test).unwrap();
        }
        let hello = storage.get("Hello").unwrap();

        assert_eq!(hello.as_ref(), test_dataset.get(0))
    }

    #[test]
    fn remove_item() {
        let storage = get_test_storage();
        let test_dataset = test_dataset();

        for test in &test_dataset{
            storage.insert(&test.string, test).unwrap();
        }
        let _ = storage.delete("Hello").unwrap();
        let hello = storage.get("Hello").unwrap();
        assert_eq!(hello, None)
    }

    #[test]
    fn update_item() {
        let storage = get_test_storage();
        let test_dataset = test_dataset();

        for test in &test_dataset{
            storage.insert(&test.string, test).unwrap();
        }
        let _ = storage.update("Hello", |x| {
            assert_eq!(x, &test_dataset[0]);
            x.number = 10
        }).unwrap();

        let hello = storage.get("Hello").unwrap();

        assert_ne!(hello.as_ref(), test_dataset.get(0))
    }

    #[test]
    fn select_item() {
        let storage = get_test_storage();
        let test_dataset = test_dataset();

        for test in &test_dataset{
            storage.insert(&test.string, test).unwrap();
        }

        let hello = storage.select(Query::from_str("SELECT * WHERE number = 10").unwrap()).unwrap();
        let expected = test_dataset.get(1).unwrap();

        assert!(hello.rows().eq([[
            Value::Number(expected.number.into()),
            Value::String(expected.string.to_string()),
            Value::DateTime(expected.date_time)
        ]]));

    }

    fn get_test_storage<T: Serialize + for<'a> Deserialize<'a>>() -> Storage<T> {
        let tempdir = tempdir().unwrap();

        Storage::open(&tempdir).unwrap()
    }
}