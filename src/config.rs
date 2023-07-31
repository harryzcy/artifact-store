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

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::{remove_var, set_var};

    #[test]
    fn load_config() {
        {
            remove_var("DATA_DIR");
            remove_var("ROCKSDB_PATH");
            let config = load();
            assert_eq!(config.data_dir, "/data");
            assert_eq!(config.rocksdb_path, "/data/rocksdb");
        }

        {
            set_var("DATA_DIR", "/data");
            remove_var("ROCKSDB_PATH");
            let config = load();
            assert_eq!(config.data_dir, "/data");
            assert_eq!(config.rocksdb_path, "/data/rocksdb");
        }

        {
            set_var("DATA_DIR", "/data");
            set_var("ROCKSDB_PATH", "/etc/rocksdb");
            let config = load();
            assert_eq!(config.data_dir, "/data");
            assert_eq!(config.rocksdb_path, "/etc/rocksdb");
        }
    }
}
