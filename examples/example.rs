use hurrahdb::{AofConfig, Config, Storage, Type};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct TestModel1 {
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct TestModel2 {
    value: String,
}

#[tokio::main]
async fn main() {
    let storage = match Storage::new(Some(Config {
        aof_config: Some(AofConfig {
            sync_time: 100,
            file_name: "example_file".to_string(),
        }),
        persistance_type: Type::Aof,
    })) {
        Ok(s) => s,
        Err(err) => {
            panic!("unable to create storage {}", err)
        }
    };

    let model1 = TestModel1 {
        name: "some-value-1".to_string(),
    };
    let model2 = TestModel2 {
        value: "some-value-2".to_string(),
    };

    if let Err(err) = storage.set("some-key-1".to_string(), &model1) {
        panic!("unable to store {}", err);
    }

    if let Err(err) = storage.set("some-key-2".to_string(), &model2) {
        panic!("unable to store {}", err);
    }

    let result1: Option<TestModel1> = match storage.get("some-key-1".to_string()) {
        Ok(r) => r,
        Err(err) => {
            panic!("unable to get result {}", err)
        }
    };

    println!("Result1 {:?}", result1);

    let result2: Option<TestModel2> = match storage.get("some-key-2".to_string()) {
        Ok(r) => r,
        Err(err) => {
            panic!("unable to get result {}", err)
        }
    };

    println!("Result2 {:?}", result2)
}
