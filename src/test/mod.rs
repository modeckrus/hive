#[cfg(test)]
pub mod test_utils {
    use std::sync::Arc;

    use crate::*;
    #[derive(
        Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash,
    )]
    pub(crate) struct Hello {
        pub(crate) world: Arc<str>,
        pub(crate) sus: String,
        pub(crate) int: i32,
    }

    impl Hello {
        pub(crate) fn new(world: Arc<str>, sus: String, int: i32) -> Self {
            Self { world, sus, int }
        }
    }

    impl Default for Hello {
        fn default() -> Self {
            Self {
                world: Arc::from("world"),
                sus: String::from("sus"),
                int: 0,
            }
        }
    }
    pub(crate) fn test_db_file_path() -> std::path::PathBuf {
        let dir = std::env::temp_dir().join("embed_db_test");
        dir
    }
    pub(crate) fn remove_test_db(){
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
    }
    pub(crate) fn test_db_file() -> HiveBox<Hello> {
        let dir = test_db_file_path();
        HiveBox::<Hello>::new(dir).unwrap()
    }

    pub(crate) fn test_db_memory() -> HiveBox<Hello> {
        HiveBox::<Hello>::memory().unwrap()
    }

    pub(crate) fn sled_memory() -> sled::Db {
        let config = sled::Config::new().temporary(true);
        config.open().unwrap()
    }
}
