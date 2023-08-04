use std::env::var;

pub struct Config {
    /// The base directory where all data is stored.
    pub data_path: String,
    /// The path to the rocksdb database, default to $data_path/rocksdb.
    pub rocksdb_path: String,
    /// The path to the artifacts directory, default to $data_path/artifacts.
    pub artifact_path: String,
}

pub fn load() -> Config {
    let data_path = match var("DATA_PATH") {
        Ok(dir) => dir,
        Err(_) => "/data".to_string(),
    };
    let rocksdb_path = match var("ROCKSDB_PATH") {
        Ok(dir) => dir,
        Err(_) => format!("{}/rocksdb", data_path).to_string(),
    };
    let artifact_path = match var("ARTIFACTS_PATH") {
        Ok(dir) => dir,
        Err(_) => format!("{}/artifacts", data_path).to_string(),
    };

    Config {
        data_path,
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
            remove_var("DATA_PATH");
            remove_var("ROCKSDB_PATH");
            remove_var("ARTIFACTS_PATH");
            let config = load();
            assert_eq!(config.data_path, "/data");
            assert_eq!(config.rocksdb_path, "/data/rocksdb");
            assert_eq!(config.artifact_path, "/data/artifacts");
        }

        {
            set_var("DATA_PATH", "/data");
            remove_var("ROCKSDB_PATH");
            remove_var("ARTIFACTS_PATH");
            let config = load();
            assert_eq!(config.data_path, "/data");
            assert_eq!(config.rocksdb_path, "/data/rocksdb");
            assert_eq!(config.artifact_path, "/data/artifacts");
        }

        {
            set_var("DATA_PATH", "/data");
            set_var("ROCKSDB_PATH", "/etc/rocksdb");
            remove_var("ARTIFACTS_PATH");
            let config = load();
            assert_eq!(config.data_path, "/data");
            assert_eq!(config.rocksdb_path, "/etc/rocksdb");
            assert_eq!(config.artifact_path, "/data/artifacts");
        }

        {
            set_var("DATA_PATH", "/data");
            remove_var("ROCKSDB_PATH");
            set_var("ARTIFACTS_PATH", "/etc/artifacts");
            let config = load();
            assert_eq!(config.data_path, "/data");
            assert_eq!(config.rocksdb_path, "/data/rocksdb");
            assert_eq!(config.artifact_path, "/etc/artifacts");
        }

        {
            set_var("DATA_PATH", "/data");
            set_var("ROCKSDB_PATH", "/etc/rocksdb");
            set_var("ARTIFACTS_PATH", "/etc/artifacts");
            let config = load();
            assert_eq!(config.data_path, "/data");
            assert_eq!(config.rocksdb_path, "/etc/rocksdb");
            assert_eq!(config.artifact_path, "/etc/artifacts");
        }
    }
}
