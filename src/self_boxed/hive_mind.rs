use std::fmt::Debug;

use crate::{HiveBoxable, HiveError};

#[derive(Debug, Clone)]
pub struct HiveMind {
    pub sled: sled::Db,
}

impl HiveMind {
    pub fn new(sled: sled::Db) -> Self {
        Self { sled }
    }

    pub fn get<T, N>(&self, name: N) -> Result<T, HiveError>
    where
        T: HiveBoxable,
        N: AsRef<[u8]>,
    {
        let bytes = self.sled.get(name)?.ok_or(HiveError::None)?;
        let value = pot::from_slice::<T>(&bytes)?;
        Ok(value)
    }

    pub fn set<T, N>(&self, name: N, value: &T) -> Result<(), HiveError>
    where
        T: HiveBoxable,
        N: AsRef<[u8]>,
    {
        let bytes = pot::to_vec(value)?;
        self.sled.insert(name, bytes)?;
        Ok(())
    }

    pub fn set_bytes<N>(&self, name: N, value: &[u8]) -> Result<(), HiveError>
    where
        N: AsRef<[u8]>,
    {
        self.sled.insert(name, value)?;
        Ok(())
    }

    pub fn iter<T: HiveBoxable>(&self) -> impl Iterator<Item = T> {
        self.sled
            .iter()
            .map(|result| result.ok())
            .flatten()
            .map(|(_, bytes)| {
                let hello = pot::from_slice::<T>(&bytes).unwrap();
                hello
            })
            .into_iter()
    }

    pub fn iter_with_keys<T: HiveBoxable>(&self) -> impl Iterator<Item = (Vec<u8>, T)> {
        self.sled
            .iter()
            .map(|result| result.ok())
            .flatten()
            .map(|(key, bytes)| {
                if let Some(v) = pot::from_slice::<T>(&bytes).ok() {
                    Some((key.to_vec(), v))
                } else {
                    None
                }
            })
            .flatten()
            .into_iter()
    }
}
