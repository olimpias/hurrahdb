use std::fs::File;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::io::{self, Write};
use tokio::time::{self, Duration};
use std::ops::Deref;
use std::fs::OpenOptions;
use std::io::BufRead;
use std::error;

pub struct Aof {
    filename: String,
    safe_file: Arc<Mutex<File>>,
    sync_time: u64
}

#[derive(PartialEq)]
enum ActionType {
    Set,
    Del
} 

impl ActionType {
    fn as_str(&self) -> &'static str {
        match self {
            ActionType::Set => "Set",
            ActionType::Del => "Del"
        }
    }
    fn from(input: &str) -> Option<ActionType> {
        match input {
            "Set" => Some(ActionType::Set),
            "Del" => Some(ActionType::Del),
            _ => None
        }
    }
}

type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

impl Aof {
    // Returns a new AOF with ARC and collects key value pairs from `filename`.
    // 
    // In addition that the method triggers a subthread period of `sync_time` to flush written values into file.
    pub fn new(filename: String,sync_time: u64) -> Result<(HashMap<String, String>,Aof)> {
        let f = OpenOptions::new().create(true).write(true).append(true).open(&filename)?;
        
        let aof = Self{filename, sync_time, safe_file: Arc::new(Mutex::new(f))};
        let map = aof.read_file()?;
        aof.flush();
        Ok((map, aof))
    }


    // Reads the filename that passed during constructor and creates the initial hashmap.
    fn read_file(&self) -> Result<HashMap<String, String>> {
        let mut map = HashMap::new();
        let file = File::open(&self.filename)?;
        let reader = io::BufReader::new(file);
        let mut current_operation = ActionType::Del;
        let mut key = "".to_string();
        let mut value = "".to_string();
        let mut counter = 0;
        let mut process_data = false;
        let mut max_counter = 0;
        for line in reader.lines() {
            let line_value = match line {
                Ok(l) => l,
                Err(err) => {panic!("unable to read file {}", err)}
            };
            match counter {
                0=> {
                    current_operation = match ActionType::from(line_value.as_str()){
                        Some(val) => val,
                        None => panic!("action type is invalid, corrupted file")
                    };
                    match current_operation {
                        ActionType::Del => max_counter = 2,
                        ActionType::Set => max_counter = 3
                    }
                },
                1=> {
                    key = line_value;
                    if current_operation == ActionType::Del {
                        process_data = true;
                    }
                    if key.len() == 0 {
                        panic!("key value can not be empty, corrupted file")
                    }
                },
                2=>{
                    value = line_value;
                    process_data = true;
                },
                _=>{
                    panic!("corrupted file");
                }
            }
            if process_data {
                match current_operation {
                    ActionType::Set => {
                        map.insert(key.clone(), value.clone());
                    }
                    ActionType::Del => {
                        map.remove(&key);
                    }
                }
                process_data = false;
            }
            
            counter = (counter + 1) % max_counter;
        }
        return Ok(map);
    }

    // Flushes buffered values into file async.
    fn flush(&self) {
        let sync_time = self.sync_time;
        let safe_file_clone = self.safe_file.clone();
        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(sync_time));
            loop {
                interval.tick().await;
                match safe_file_clone.lock().unwrap().flush(){
                    Ok(()) => {},
                    Err(err) => {
                        println!("{}", err);
                    }
                }
                return;
            }
        });
    }

    // Forms a Set command value and pushes into file buffer
    pub fn set(&self, key: String, val: String) {
        let lines = format!("{}\n{}\n{}\n", ActionType::Set.as_str(), key, val);
        match self.safe_file.clone().lock().unwrap().write(lines.as_bytes()){
            Ok(_) => {},
            Err(err) => {
                println!("{}", err);
            }
        }
    }

    // Forms a Del command value and pushes into file buffer
    pub fn del(&self, key: String) {
        let lines = format!("{}\n{}\n", ActionType::Del.as_str(), key);
        match self.safe_file.clone().lock().unwrap().write(lines.as_bytes()){
            Ok(_) => {},
            Err(err) => {
                println!("{}", err);
            }
        }    
    }
}

impl Deref for Aof {
    type Target = Arc<Mutex<File>>;

    fn deref(&self) -> &Arc<Mutex<File>> {
        match self.safe_file.clone().try_lock().unwrap().flush() {
            Ok(_) => {},
            Err(err) => {
                println!("{}", err); 
            }
        }
        &self.safe_file
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use super::*;

      #[tokio::test]
        async fn validate_write() {
        let (_, aof) = match Aof::new("somefile".to_string(), 1){
            Ok((map, aof)) => {
                (map, aof)
            },
            Err(err) => {
                panic!("{}", err)
            }
        };
        aof.set("some-key-1".to_string(), "some-val".to_string());
        aof.set("some-key-2".to_string(), "some-val".to_string());
        aof.del("some-key-2".to_string());
        tokio::time::sleep(Duration::from_secs(2)).await;
        let contents = fs::read_to_string("somefile")
        .expect("Should have been able to read the file");
        assert_eq!(contents, "Set\nsome-key-1\nsome-val\nSet\nsome-key-2\nsome-val\nDel\nsome-key-2\n");
        fs::remove_file("somefile").expect("unable to delete file");
    }

    #[tokio::test]
        async fn validate_reading_from_aof_file() {
            fs::write("somefile-write", "Set\nsome-key-1\nsome-val\nSet\nsome-key-2\nsome-val\nDel\nsome-key-2\n").expect("unable to write into file");
            let (map_value, _) = match Aof::new("somefile-write".to_string(), 1){
                Ok((map, aof)) => {
                    (map, aof)
                },
                Err(err) => {
                    panic!("{}", err)
                }
            };
            assert_eq!(map_value["some-key-1"], "some-val");
            fs::remove_file("somefile-write").expect("unable to delete file");
        }
}
