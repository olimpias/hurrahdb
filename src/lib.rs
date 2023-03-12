mod persistance;

use serde::{de, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::{error::Error, fmt};


#[derive(Debug)]
pub struct ConfigMissing {
    persistance_type: Type,
}

#[derive(PartialEq, Debug)]
pub enum Type {
    None,
    Aof,
}

impl Error for ConfigMissing {}

impl fmt::Display for ConfigMissing {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "configuration is not set for {:?} persistance type",
            self.persistance_type
        )
    }
}

pub struct Config {
    pub aof_config: Option<AofConfig>,
    pub persistance_type: Type,
}

pub struct AofConfig {
    pub sync_time: u64,
    pub file_name: String,
}

pub struct Storage {
    cache: Arc<RwLock<HashMap<String, String>>>,
    storage: Arc<dyn persistance::Persist>,
}

impl Storage {
    // Return a storage without any persitance config
    fn new_cache_without_persistance() -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            storage: Arc::new(persistance::Empty {}),
        }
    }
    // Return a storage with persistance based on the configuration that is passed
    pub fn new(config: Option<Config>) -> persistance::Result<Self> {
        if let Some(c) = config {
            match c.persistance_type {
                Type::None => {
                    return Ok(Storage::new_cache_without_persistance());
                }
                Type::Aof => match c.aof_config {
                    Some(config) => {
                        let (read_map, storage) =
                            persistance::aof::Storage::new(config.file_name, config.sync_time)?;
                        return Ok(Self {
                            cache: Arc::new(RwLock::new(read_map)),
                            storage: Arc::new(storage),
                        });
                    }
                    None => {
                        return Err(Box::new(ConfigMissing {
                            persistance_type: c.persistance_type,
                        }));
                    }
                },
            }
        };

        Ok(Storage::new_cache_without_persistance())
    }

    pub fn set<T: Serialize>(&self, key: String, val: &T) -> persistance::Result<()> {
        let serilized_val = serde_json::to_string(&val).unwrap(); // todo
        self.storage.clone().set(&key, &serilized_val);
        self.cache
            .clone()
            .try_write()
            .unwrap()
            .insert(key, serilized_val);
        Ok(())
    }
    pub fn del(&self, key: &String) {
        self.cache.clone().try_write().unwrap().remove(key);
        self.storage.clone().del(key);
    }

    pub fn get<T: de::DeserializeOwned>(&self, key: String) -> persistance::Result<Option<T>> {
        let binding = self.cache.clone();
        let read_lock = binding.try_read().unwrap();
        let ref_val = read_lock.get(&key);
        if let Some(v) = ref_val {
            let result: T = serde_json::from_str(v)?;
            return Ok(Some(result));
        }

        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use std::fs;
    use tokio::time::Duration;

    #[derive(Serialize, Deserialize)]
    struct TestModel {
        value: String,
    }

    #[tokio::test]
    async fn set_get_key_value_from_cache_without_persistance() {
        let storage = match Storage::new(None) {
            Ok(storage) => storage,
            Err(err) => {
                panic!("unable to create storage {}", err)
            }
        };
        match storage.set(
            "some-key".to_string(),
            &TestModel {
                value: "some-value".to_string(),
            },
        ) {
            Ok(()) => {}
            Err(err) => {
                panic!("unable to set in cache {}", err)
            }
        }

        let result_option: Option<TestModel> = match storage.get("some-key".to_string()) {
            Ok(result) => result,
            Err(err) => {
                panic!("unable to get from cache {}", err)
            }
        };

        match result_option {
            Some(result) => {
                assert_eq!("some-value".to_string(), result.value)
            }
            None => {
                panic!("unable to find data in cache")
            }
        }
    }

    #[tokio::test]
    async fn set_get_key_value_from_cache_with_aof() {
        let storage = match Storage::new(Some(Config {
            aof_config: Some(AofConfig {
                sync_time: 100,
                file_name: "memory-cache-test-1".to_string(),
            }),
            persistance_type: Type::Aof,
        })) {
            Ok(storage) => storage,
            Err(err) => {
                panic!("unable to create storage {}", err)
            }
        };
        match storage.set(
            "some-key".to_string(),
            &TestModel {
                value: "some-value".to_string(),
            },
        ) {
            Ok(()) => {}
            Err(err) => {
                panic!("unable to set in cache {}", err)
            }
        }

        let result_option: Option<TestModel> = match storage.get("some-key".to_string()) {
            Ok(result) => result,
            Err(err) => {
                panic!("unable to get from cache {}", err)
            }
        };

        match result_option {
            Some(result) => {
                assert_eq!("some-value".to_string(), result.value)
            }
            None => {
                panic!("unable to find data in cache")
            }
        }

        tokio::time::sleep(Duration::from_millis(100)).await;
        let contents = fs::read_to_string("memory-cache-test-1")
            .expect("Should have been able to read the file");
        assert_eq!(contents, "Set\nsome-key\n{\"value\":\"some-value\"}\n");
        fs::remove_file("memory-cache-test-1").expect("unable to delete file");
    }
}
