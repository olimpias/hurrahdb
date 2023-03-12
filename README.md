# hurrahdb

Hurrahdb is an inmemory key value store with an option of persistance in Rust. Currently only supports AOF option to persist.

Persistance of the data using AOF is async and flushing data into db based on `sync_time`. Unit of the `sync_time` in milliseconds.

## Usage

While caching/storing a data, key needs to be `string` type and value needs to be a struct or enum that derives `Serialize` and `Deserialize` from `serde` library. An example model below

```rust
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct DummyStruct {
    value: String,
}
```

### Only In-memory

When a storage is constracted without any config, it will not initilize any persistance logic.

```rust
// define a storage object with None
let storage = match Storage::new(None) {
    Ok(storage) => storage,
    Err(err) => {
        // Handle the error
    }
};

// Cache the `DummyStruct` struct with key "some-key"
match storage.set(
    "some-key".to_string(),
    &DummyStruct {
        value: "some-value".to_string(),
    },
) {
    Ok(()) => {}
    Err(err) => {
        // Handle the error
    }
}

// Fetch the `DummyStruct` data using key "some-key"
let result_option: Option<DummyStruct> = match storage.get("some-key".to_string()) {
    Ok(result) => result,
    Err(err) => {
        // Handle the error
    }
};
```

### Inmemory With AOF

When a storage is created with AOF config, it reads the input file and precreates the hashmap with the data in the file. In addition to that creates a background job to flush data into disk based on `sync_time` value.

**Note**:

* Since the data flushing async, it does not grantee the persistance. You might loss your data during shotdown/crash states of the app.
* **Requires** `tokio run time`

**Example usage** [here](examples/example.rs)

```rust

// Define a storage with AOF config. Flushes data into file every 100ms.
let storage = match Storage::new(Some(Config {
    aof_config: Some(AofConfig {
        sync_time: 100,
        file_name: "memory-cache-test-1".to_string(),
    }),
    persistance_type: persistance::Type::AOF,
})) {
    Ok(storage) => storage,
    Err(err) => {
        // Handle the error
    }
};

// Cache the `DummyStruct` struct with key "some-key"
match storage.set(
    "some-key".to_string(),
    &DummyStruct {
        value: "some-value".to_string(),
    },
) {
    Ok(()) => {}
    Err(err) => {
        // Handle the error
    }
}

// Fetch the `DummyStruct` data using key "some-key"
let result_option: Option<DummyStruct> = match storage.get("some-key".to_string()) {
    Ok(result) => result,
    Err(err) => {
        // Handle the error
    }
};
```
