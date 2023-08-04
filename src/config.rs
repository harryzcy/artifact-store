use std::env::var;

pub struct Config {
    /// The base directory where all data is stored.
    pub data_dir: String,
    /// The path to the rocksdb database, default to $data_dir/rocksdb.
    pub rocksdb_path: String,
    /// The path to the artifacts directory, default to $data_dir/artifacts.
    pub artifact_path: String,
}

pub fn load() -> Config {
    let data_dir = match var("DATA_PATH") {
        Ok(dir) => dir,
        Err(_) => "/data".to_string(),
    };
    let rocksdb_path = match var("ROCKSDB_PATH") {
        Ok(dir) => dir,
        Err(_) => format!("{}/rocksdb", data_dir).to_string(),
    };
    let artifact_path = match var("ARTIFACT_PATH") {
        Ok(dir) => dir,
        Err(_) => format!("{}/artifacts", data_dir).to_string(),
    };

    Config {
        data_dir,
        rocksdb_path,
        artifact_path,
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
