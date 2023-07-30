use std::env::var;

pub struct Config {
    pub data_dir: String,
    pub rocksdb_path: String,
}

pub fn load() -> Config {
    let data_dir = match var("DATA_DIR") {
        Ok(dir) => dir,
        Err(_) => "/data".to_string(),
    };
    let rocksdb_path = match var("ROCKSDB_PATH") {
        Ok(dir) => dir,
        Err(_) => format!("{}/rocksdb", data_dir).to_string(),
    };

    Config {
        data_dir,
        rocksdb_path,
    }
}
