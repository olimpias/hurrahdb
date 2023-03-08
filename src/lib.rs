mod persistance;

use std::sync::{Arc, RwLock};
use std::collections::HashMap;

pub struct Config {
    pub aof_sync_time: u64,
    pub aof_file_name: String,
    pub persistance_type: persistance::Type
}

pub struct Storage {
    cache: Arc<RwLock<HashMap<String, String>>>,
    persistance_type: persistance::Type,
    storage: Arc<dyn persistance::Persist>
}

impl Storage {
    // Return a storage without any persitance config
    fn new_cache_without_persistance() -> Self{
        Self { cache: Arc::new(RwLock::new(HashMap::new())), persistance_type: persistance::Type::NONE, storage: Arc::new(persistance::Empty{}) }
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
                    return Ok(Self { cache: Arc::new(RwLock::new(read_map)), persistance_type: c.persistance_type, storage: Arc::new(storage)});
                }
            }
        };

        Ok(Storage::new_cache_without_persistance())
    }

    pub fn set(&self, key: String, val: String){
        self.cache.clone().try_write().unwrap().insert(key.clone(), val.clone());
        self.storage.clone().set(key, val);
    }
    pub fn del(&self, key: String){
        self.cache.clone().try_write().unwrap().remove(&key);
        self.storage.clone().del(key);
    }

    pub fn get(&self, key: String) -> Option<String> {
        self.clone().cache.try_read().unwrap().get(&key).cloned()
    }
    
}



