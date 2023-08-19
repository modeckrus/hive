use std::fmt::Debug;

mod test;
pub mod self_boxed;

pub use self_boxed::{HiveNamed, SelfHiveBoxed, hive_mind::HiveMind};
pub trait HiveBoxable: serde::de::DeserializeOwned + serde::Serialize + Debug {}

impl<T: serde::de::DeserializeOwned + serde::Serialize + Debug> HiveBoxable for T {}

pub struct HiveBox<T: HiveBoxable> {
    pub sled: sled::Db,
    pub path: Option<std::path::PathBuf>,
    _phantom: std::marker::PhantomData<T>,
}

//error for get from db
#[derive(Debug, thiserror::Error)]
pub enum HiveError {
    #[error("sled error: {0}")]
    Sled(#[from] sled::Error),
    #[error("bincode error: {0}")]
    Bincode(#[from] bincode::Error),
    #[error("no value found")]
    None,
}

impl<T: HiveBoxable> HiveBox<T> {
    pub fn new(path: impl AsRef<std::path::Path>) -> Result<Self, sled::Error> {
        let path = path.as_ref();
        Ok(Self {
            sled: sled::open(path.clone())?,
            path: Some(path.to_path_buf()),
            _phantom: std::marker::PhantomData,
        })
    }

    pub fn memory() -> Result<Self, sled::Error> {
        Ok(Self {
            sled: sled::Config::new().temporary(true).open()?,
            path: None,
            _phantom: std::marker::PhantomData,
        })
    }

    pub fn insert(&self, key: &str, value: T) -> Result<(), sled::Error> {
        let bytes = bincode::serialize(&value).unwrap();
        self.sled.insert(key, bytes)?;
        Ok(())
    }

    pub fn get(&self, key: &str) -> Result<T, HiveError> {
        let bytes = self.sled.get(key)?.ok_or(HiveError::None)?;
        let value = bincode::deserialize::<T>(&bytes)?;
        Ok(value)
    }

    pub fn iter(&self) -> impl Iterator<Item = T> {
        self.sled
            .iter()
            .map(|result| result.ok())
            .flatten()
            .map(|(_, bytes)| {
                let hello = bincode::deserialize::<T>(&bytes).unwrap();
                hello
            })
            .into_iter()
    }

    pub fn vec(&self) -> Vec<T> {
        self.iter().collect()
    }

    pub fn remove(&self, key: &str) -> Result<(), sled::Error> {
        self.sled.remove(key)?;
        Ok(())
    }
}

impl<T: std::hash::Hash + HiveBoxable> HiveBox<T> {
    pub fn add(&self, value: T) -> Result<(), HiveError> {
        let bytes = bincode::serialize(&value)?;
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        value.hash(&mut hasher);
        let hash = std::hash::Hasher::finish(&hasher);
        self.sled.insert(hash.to_le_bytes(), bytes)?;
        Ok(())
    }

    pub fn exact(&self, value: &T) -> Result<T, HiveError> {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        value.hash(&mut hasher);
        let hash = std::hash::Hasher::finish(&hasher);
        let bytes = self.sled.get(hash.to_le_bytes())?.ok_or(HiveError::None)?;
        let value = bincode::deserialize::<T>(&bytes)?;
        Ok(value)
    }

    pub fn remove_dublicate(&self, value: &T) -> Result<(), HiveError> {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        value.hash(&mut hasher);
        let hash = std::hash::Hasher::finish(&hasher);
        self.sled.remove(hash.to_le_bytes())?;
        Ok(())
    }
}
#[cfg(test)]
mod test_hive_mind {
    use crate::test::test_utils::*;

    use super::*;
    use std::sync::Arc;
    
    #[test]
    fn test_db_with_file() {
        let dir = test_db_file_path();
        //remove dir
        match std::fs::remove_dir_all(dir) {
            Ok(_) => {}
            Err(e) => {
                if e.kind() != std::io::ErrorKind::NotFound {
                    panic!("error removing dir: {}", e);
                }
            }
        }
        let db = test_db_file();

        let hello = Hello::default();
        db.insert("hello", hello.clone()).unwrap();
        let hello2 = Hello::new(Arc::from("world"), String::from("sus"), 1);
        db.insert("hello2", hello2.clone()).unwrap();
        let mut iter = db.iter();
        assert_eq!(iter.next().unwrap(), hello);
        assert_eq!(iter.next().unwrap(), hello2);
    }

    #[test]
    fn test_db_restore() {
        let dir = test_db_file_path();
        //remove dir
        match std::fs::remove_dir_all(dir) {
            Ok(_) => {}
            Err(e) => {
                if e.kind() != std::io::ErrorKind::NotFound {
                    panic!("error removing dir: {}", e);
                }
            }
        }
        {
            let db = test_db_file();
            db.insert("hello", Hello::default()).unwrap();
        }
        {
            let db = test_db_file();
            let hello = db.get("hello").unwrap();
            assert_eq!(hello, Hello::default());
        }
    }

    #[test]
    fn test_insert() {
        let db = test_db_memory();
        let hello = Hello::default();
        db.insert("hello", hello.clone()).unwrap();
        let hello2 = db.get("hello").unwrap();
        assert_eq!(hello, hello2);
    }

    #[test]
    fn test_iter() {
        let db = test_db_memory();
        let hello = Hello::default();
        db.insert("hello", hello.clone()).unwrap();
        let hello2 = Hello::new(Arc::from("world"), String::from("sus"), 1);
        db.insert("hello2", hello2.clone()).unwrap();
        let mut iter = db.iter();
        assert_eq!(iter.next().unwrap(), hello);
        assert_eq!(iter.next().unwrap(), hello2);
    }

    #[test]
    fn test_vec() {
        let db = test_db_memory();
        let hello = Hello::default();
        db.insert("hello", hello.clone()).unwrap();
        let hello2 = Hello::new(Arc::from("world"), String::from("sus"), 1);
        db.insert("hello2", hello2.clone()).unwrap();
        let vec = db.vec();
        assert_eq!(vec[0], hello);
        assert_eq!(vec[1], hello2);
    }

    #[test]
    fn test_remove() {
        let db = test_db_memory();
        let hello = Hello::default();
        db.insert("hello", hello.clone()).unwrap();
        db.remove("hello").unwrap();
        assert!(db.get("hello").is_err());
    }

    #[test]
    fn test_add() {
        let db = test_db_memory();
        let hello = Hello::default();
        db.add(hello.clone()).unwrap();
        let hello2 = Hello::new(Arc::from("world"), String::from("sus"), 1);
        db.add(hello2.clone()).unwrap();
        let mut iter = db.iter();
        assert_eq!(iter.next().unwrap(), hello);
        assert_eq!(iter.next().unwrap(), hello2);
    }

    #[test]
    fn test_exact() {
        let db = test_db_memory();
        let hello = Hello::default();
        db.add(hello.clone()).unwrap();
        let hello2 = Hello::new(Arc::from("world"), String::from("sus"), 1);
        db.add(hello2.clone()).unwrap();
        let hello3 = Hello::new(Arc::from("world"), String::from("sus"), 1);
        let hello4 = db.exact(&hello3).unwrap();
        assert_eq!(hello4, hello2);
    }
}
