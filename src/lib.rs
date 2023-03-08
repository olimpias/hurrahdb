mod persistance;

use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use serde::{de, Serialize};

pub struct Config {
    pub aof_sync_time: u64,
    pub aof_file_name: String,
    pub persistance_type: persistance::Type
}

pub struct Storage {
    cache: Arc<RwLock<HashMap<String, String>>>,
    storage: Arc<dyn persistance::Persist>
}

impl Storage {
    // Return a storage without any persitance config
    fn new_cache_without_persistance() -> Self{
        Self { cache: Arc::new(RwLock::new(HashMap::new())), storage: Arc::new(persistance::Empty{}) }
    }
    // Return a storage with persistance based on the configuration that is passed
    pub fn new(config: Option<Config>) -> persistance::Result<Self> {
        if let Some(c) = config {
            match c.persistance_type {
                persistance::Type::NONE => {
                    return Ok(Storage::new_cache_without_persistance());
                }
                persistance::Type::AOF => {
                    let (read_map, storage) = persistance::aof::Storage::new(c.aof_file_name, c.aof_sync_time)?;
                    return Ok(Self { cache: Arc::new(RwLock::new(read_map)), storage: Arc::new(storage)});
                }
            }
        };

        Ok(Storage::new_cache_without_persistance())
    }

    pub fn set<T: Serialize>(&self, key: String, val: &T) -> persistance::Result<()>{
        let serilized_val = serde_json::to_string(&val).unwrap();// todo
        self.storage.clone().set(&key, &serilized_val);
        self.cache.clone().try_write().unwrap().insert(key, serilized_val);
        Ok(())
    }
    pub fn del(&self, key: &String){
        self.cache.clone().try_write().unwrap().remove(key);
        self.storage.clone().del(key);
    }

    pub fn get<T: de::DeserializeOwned>(&self, key: String) -> persistance::Result<Option<T>> {
        let binding = self.clone().cache.try_read().unwrap();
        let ref_val = binding.get(&key);
        match ref_val {
            Some(v) => {
                let result: T = serde_json::from_str(v)?;
                return Ok(Some(result));
            }
            _ => {}
        }
        return Ok(None);
    }
    
}

#[cfg(test)]
mod tests {
    use std::fs;
    use super::*;

      #[test]
        fn validate_write() {
    }

    #[test]
        fn validate_reading_from_aof_file() {
        }
}
