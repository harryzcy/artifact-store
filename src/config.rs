use std::env::var;

pub struct Config {
    /// The path to the rocksdb database, default to $DATA_PATH/rocksdb.
    pub rocksdb_path: String,
    /// The path to the artifacts directory, default to $DATA_PATH/artifacts.
    pub artifact_path: String,
}

pub fn load() -> Config {
    let data_path = match var("DATA_PATH") {
        Ok(dir) => dir,
        Err(_) => "/data".to_string(),
    };
    let rocksdb_path = match var("ROCKSDB_PATH") {
        Ok(dir) => dir,
        Err(_) => format!("{data_path}/rocksdb").to_string(),
    };
    let artifact_path = match var("ARTIFACTS_PATH") {
        Ok(dir) => dir,
        Err(_) => format!("{data_path}/artifacts").to_string(),
    };

    Config {
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
            assert_eq!(config.rocksdb_path, "/data/rocksdb");
            assert_eq!(config.artifact_path, "/data/artifacts");
        }

        {
            set_var("DATA_PATH", "/data");
            remove_var("ROCKSDB_PATH");
            remove_var("ARTIFACTS_PATH");
            let config = load();
            assert_eq!(config.rocksdb_path, "/data/rocksdb");
            assert_eq!(config.artifact_path, "/data/artifacts");
        }

        {
            set_var("DATA_PATH", "/data");
            set_var("ROCKSDB_PATH", "/etc/rocksdb");
            remove_var("ARTIFACTS_PATH");
            let config = load();
            assert_eq!(config.rocksdb_path, "/etc/rocksdb");
            assert_eq!(config.artifact_path, "/data/artifacts");
        }

        {
            set_var("DATA_PATH", "/data");
            remove_var("ROCKSDB_PATH");
            set_var("ARTIFACTS_PATH", "/etc/artifacts");
            let config = load();
            assert_eq!(config.rocksdb_path, "/data/rocksdb");
            assert_eq!(config.artifact_path, "/etc/artifacts");
        }

        {
            set_var("DATA_PATH", "/data");
            set_var("ROCKSDB_PATH", "/etc/rocksdb");
            set_var("ARTIFACTS_PATH", "/etc/artifacts");
            let config = load();
            assert_eq!(config.rocksdb_path, "/etc/rocksdb");
            assert_eq!(config.artifact_path, "/etc/artifacts");
        }
    }
}
