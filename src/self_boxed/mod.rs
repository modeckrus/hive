pub mod hive_mind;
use std::sync::Arc;

use crate::{HiveBoxable, HiveError};

use self::hive_mind::HiveMind;
#[macro_export]
macro_rules! get {
    ($v:expr) => {
        $v.value.read().unwrap()
    };
}
#[macro_export]
macro_rules! initialize_hive_mind {
    ($e:expr) => {
        lazy_static::lazy_static! {
            pub static ref HIVE_MIND: crate::HiveMind = crate::HiveMind::new($e);
        }
    };
}



#[macro_export]
macro_rules! initialize_self_hive_boxed {
    ($v:vis $name:ident, $ty:ty, $e:expr) => {
        lazy_static::lazy_static! {
            pub static ref $name: crate::SelfHiveBoxed<$ty, &'static [u8]> =
            crate::SelfHiveBoxed::<$ty, &'static [u8]>::initialize(
                    Some(HIVE_MIND.clone()),
                    <$ty>::hive_name(),
                    // Hello::new(Arc::from("world"), "not sus".to_string(), 0)
                    $e
                )
                .unwrap();
        }
    };
}

pub struct SelfHiveBoxed<T, N>
where
    T: HiveBoxable,
    N: AsRef<[u8]>,
{
    pub hive_mind: Option<HiveMind>,
    pub name: N,
    pub value: Arc<std::sync::RwLock<T>>,
}

impl<T, N> SelfHiveBoxed<T, N>
where
    T: HiveBoxable,
    N: AsRef<[u8]> + Clone,
{
    pub fn new(hive_mind: Option<HiveMind>, name: N, value: T) -> Result<Self, HiveError> {
        let value = Arc::new(std::sync::RwLock::new(value));
        if let Some(hive_mind) = hive_mind.clone() {
            hive_mind.set(name.clone(), &value)?;
        }
        Ok(Self {
            hive_mind,
            name,
            value,
        })
    }

    // fn debug_print(hive_mind: Option<HiveMind>, text: &str) {
    //     if let Some(hive_mind) = hive_mind {
    //         for (key, value) in hive_mind.iter_with_keys::<T>() {
    //             println!("{}: {:?}{:?}", text, String::from_utf8(key), value);
    //         }
    //     }
    // }

    pub fn initialize(hive_mind: Option<HiveMind>, name: N, value: T) -> Result<Self, HiveError> {
        let bytes = bincode::serialize(&value)?;
        let rw_value = Arc::new(std::sync::RwLock::new(value));
        // Self::debug_print(hive_mind.clone(), "before initialize");
        if let Some(hive_mind) = hive_mind.clone() {
            let r = hive_mind.get::<T, N>(name.clone());
            if let Ok(new_value) = r {
                // println!(
                //     "{} already exists",
                //     std::str::from_utf8(name.as_ref()).unwrap()
                // );
                let new_value_arc = Arc::new(std::sync::RwLock::new(new_value));
                return Ok(Self {
                    hive_mind: Some(hive_mind),
                    name,
                    value: new_value_arc,
                });
                // } else if let Err(e) = r {
                //     println!("{}", e);
            }
        }

        // println!(
        //     "{} does not exist",
        //     std::str::from_utf8(name.as_ref()).unwrap()
        // );
        if let Some(hive_mind) = hive_mind.clone() {
            hive_mind.set_bytes(name.clone(), &bytes)?;
        }
        // Self::debug_print(hive_mind.clone(), "after initialize");
        Ok(Self {
            hive_mind,
            name,
            value: rw_value,
        })
    }

    pub fn get(hive_mind: HiveMind, name: N) -> Result<Self, HiveError> {
        let value = hive_mind.get::<T, N>(name.clone())?;
        let value = Arc::new(std::sync::RwLock::new(value));
        Ok(Self {
            hive_mind: Some(hive_mind),
            name,
            value,
        })
    }

    pub fn set(&self, value: T) -> Result<(), HiveError> {
        let bytes = bincode::serialize(&value)?;
        if let Some(ref hive_mind) = self.hive_mind {
            hive_mind.set_bytes(self.name.clone(), &bytes)?;
        }
        *self.value.write().unwrap() = value;
        Ok(())
    }
}

pub trait HiveNamed {
    fn hive_name() -> &'static [u8];
}

impl<T> SelfHiveBoxed<T, &[u8]>
where
    T: HiveBoxable + HiveNamed,
{
    pub fn get_named(hive_mind: HiveMind) -> Result<Self, HiveError> {
        let name = T::hive_name();
        let value = hive_mind.get::<T, _>(name.clone())?;
        let value = Arc::new(std::sync::RwLock::new(value));
        Ok(Self {
            hive_mind: Some(hive_mind),
            name,
            value,
        })
    }
    pub fn set_named(hive_mind: HiveMind, value: T) -> Result<Self, HiveError> {
        let name = T::hive_name();
        hive_mind.set(name.clone(), &value)?;
        let value = Arc::new(std::sync::RwLock::new(value));
        Ok(Self {
            hive_mind: Some(hive_mind),
            name,
            value,
        })
    }
}
#[cfg(test)]
mod test {
    use super::*;
    use crate::test::test_utils::*;
    use lazy_static::lazy_static;

    impl HiveNamed for Hello {
        fn hive_name() -> &'static [u8] {
            "hello".as_bytes()
        }
    }

    initialize_hive_mind!(sled::open(test_db_file_path()).unwrap());
    initialize_self_hive_boxed!(pub HELLO, Hello, Hello::default());

    // lazy_static! {
    //     static ref HIVE_MIND: HiveMind = HiveMind::new(sled::open(test_db_file_path()).unwrap());
    // }
    // lazy_static! {
    //     static ref HELLO: SelfHiveBoxed<Hello, &'static [u8]> =
    //         SelfHiveBoxed::<Hello, &'static [u8]>::initialize(
    //             Some(HIVE_MIND.clone()),
    //             Hello::hive_name(),
    //             // Hello::new(Arc::from("world"), "not sus".to_string(), 0)
    //             Hello::default()
    //         )
    //         .unwrap();
    // }

    #[test]
    fn remove_db() {
        remove_test_db();
    }
    #[test]
    fn test_hello_singleton() {
        {
            println!("{:?}", get!(HELLO));
            HELLO
                .set(Hello::new(Arc::from("world"), String::from("sus"), 0))
                .unwrap();
        }
        let hello = SelfHiveBoxed::initialize(
            Some(HIVE_MIND.clone()),
            Hello::hive_name(),
            Hello::default(),
        )
        .unwrap();
        println!("{:?}", get!(hello));
    }

    #[test]
    fn print_saved_hellos() {
        // let hello = HELLO.value.read().unwrap();
        // assert_eq!(hello.sus, "not sus");
        for (k, v) in HIVE_MIND.iter_with_keys::<Hello>() {
            println!("{:?}{:?}", String::from_utf8(k), v);
        }
    }

    // #[test]
    // fn test_singleton() {
    //     // {
    //     //     let hello = Hello::default();
    //     //     let hello = SelfHiveBoxed::<Hello, _>::new(HIVE_MIND.clone(), "hello", hello)
    //     //         .unwrap();
    //     // }
    //     let hello = HELLO.value.read().unwrap();
    //     assert_eq!(hello.sus, "sus");
    // }
    // #[test]
    // fn test_self_boxed() {
    //     {
    //         let hive_boxed = SelfHiveBoxed::<Hello, &[u8]>::new(
    //             HIVE_MIND.clone(),
    //             "test_self_boxed".as_bytes(),
    //             Hello::default(),
    //         )
    //         .unwrap();
    //     }
    //     let hive_boxed =
    //         SelfHiveBoxed::<Hello, &[u8]>::get(HIVE_MIND.clone(), "test_self_boxed".as_bytes())
    //             .unwrap();
    //     let hello = hive_boxed.value.read().unwrap();
    //     assert_eq!(hello.sus, "sus");
    // }

    // #[test]
    // fn test_self_named() {
    //     {
    //         let hive_boxed =
    //             SelfHiveBoxed::<Hello, &[u8]>::set_named(HIVE_MIND.clone(), Hello::default())
    //                 .unwrap();
    //     }
    //     let hive_boxed = SelfHiveBoxed::<Hello, &[u8]>::get_named(HIVE_MIND.clone()).unwrap();
    //     let hello = hive_boxed.value.read().unwrap();
    //     assert_eq!(hello.sus, "sus");
    // }

    #[test]
    fn debug_print() {
        let hellos = HIVE_MIND.iter::<Hello>();
        for hello in hellos {
            println!("{:?}", hello);
        }
    }

    #[test]
    fn test_hive_mind_store_on_disk() {
        {
            let hive_mind = HiveMind::new(sled::open(test_db_file_path()).unwrap());
            let hello = Hello::default();
            let bytes = bincode::serialize(&hello).unwrap();
            hive_mind.set_bytes(Hello::hive_name(), &bytes).unwrap();
        }
        {
            let hive_mind = HiveMind::new(sled::open(test_db_file_path()).unwrap());
            let hello = hive_mind.get::<Hello, _>(Hello::hive_name()).unwrap();
            assert_eq!(hello.sus, "sus");
        }
    }
}
