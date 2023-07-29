type TransactionDB = rocksdb::OptimisticTransactionDB;

#[allow(dead_code)]
pub enum Database {
    RocksDB(TransactionDB),
}

impl Database {
    pub fn new_rocksdb(path: &str) -> Result<Self, rocksdb::Error> {
        let db = TransactionDB::open_default(path)?;
        Ok(Database::RocksDB(db))
    }

    pub fn transaction(&self) -> Transaction<'_> {
        match self {
            Database::RocksDB(db) => Transaction::RocksDB(db.transaction()),
        }
    }
}

pub enum Transaction<'db> {
    RocksDB(rocksdb::Transaction<'db, TransactionDB>),
}

impl Transaction<'_> {
    /// Stores the repository data in the database
    /// If the repository already exists, do nothing.
    pub fn create_repo_if_not_exists(
        &self,
        time: u128,
        params: CreateRepositoryParams,
    ) -> Result<(), Error> {
        let key = format!("repo#{}#{}#{}", params.server, params.owner, params.repo);
        let value = time.to_be_bytes();

        match self {
            Transaction::RocksDB(tx) => {
                let key_bytes = key.as_bytes();
                let exists = tx.get(key_bytes)?.is_some();
                if exists {
                    return Ok(());
                }
                tx.put(key_bytes, &value)?;
            }
        }
        Ok(())
    }

    /// Store the commit data in the database.
    /// If the commit already exists, return an error.
    pub fn create_commit(&self, params: CreateCommitParams) -> Result<(), Error> {
        let commit_key = format!("commit:{}", params.commit);
        let commit_value = format!("{}/{}/{}", params.server, params.owner, params.repo);

        let repo_key = format!("repo:{}/{}", commit_value, params.commit);
        let repo_value = params.commit;

        match self {
            Transaction::RocksDB(tx) => {
                let commit_key_bytes = commit_key.as_bytes();
                let exists = tx.get(commit_key_bytes)?.is_some();
                if exists {
                    return Err(Error::Generic(format!(
                        "commit already exists: {}",
                        params.commit
                    )));
                }
                tx.put(commit_key_bytes, commit_value.as_bytes())?;
                tx.put(repo_key.as_bytes(), repo_value.as_bytes())?;
                Ok(())
            }
        }
    }

    /// Store the artifact data in the database.
    /// If the artifact already exists, return an error.
    pub fn create_artifact(&self, params: CreateArtifactParams) -> Result<(), Error> {
        let artifact_key = [
            format!("artifact:{}", params.commit).as_bytes(),
            &params.time.to_be_bytes(),
        ]
        .concat();

        match self {
            Transaction::RocksDB(tx) => {
                let exists = tx.get(&artifact_key)?.is_some();
                if exists {
                    return Err(Error::Generic(format!(
                        "artifact already exists: {}",
                        params.path
                    )));
                }

                tx.put(&artifact_key, params.commit.as_bytes())?;
                Ok(())
            }
        }
    }

    pub fn commit(self) -> Result<(), rocksdb::Error> {
        match self {
            Transaction::RocksDB(tx) => tx.commit(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorKind {
    CommitExists,
    ArtifactExists,
}

#[derive(Debug)]
pub enum Error {
    RocksDB(rocksdb::Error),
    Generic(String),
}

impl Error {
    pub fn kind(&self) -> Option<ErrorKind> {
        match self {
            Error::Generic(e) => {
                if e.contains("commit already exists") {
                    Some(ErrorKind::CommitExists)
                } else if e.contains("artifact already exists") {
                    Some(ErrorKind::ArtifactExists)
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

impl From<rocksdb::Error> for Error {
    fn from(e: rocksdb::Error) -> Self {
        Error::RocksDB(e)
    }
}

impl From<String> for Error {
    fn from(e: String) -> Self {
        Error::Generic(e)
    }
}

#[derive(Clone)]
pub struct CreateRepositoryParams<'a> {
    pub server: &'a String,
    pub owner: &'a String,
    pub repo: &'a String,
}

#[derive(Clone)]
pub struct CreateCommitParams<'a> {
    pub commit: &'a String,
    pub server: &'a String,
    pub owner: &'a String,
    pub repo: &'a String,
}

#[derive(Clone)]
pub struct CreateArtifactParams<'a> {
    pub time: &'a u128,
    pub commit: &'a String,
    pub path: &'a String,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn remove_db(path: &str) {
        let _ = std::fs::remove_dir_all(path);
    }

    #[test]
    fn test_create_commit() {
        let db = Database::new_rocksdb("data/test_create_commit").unwrap();
        let tx = db.transaction();
        let params = CreateCommitParams {
            commit: &"1234567890abcdef".to_string(),
            server: &"github.com".to_string(),
            owner: &"owner".to_string(),
            repo: &"repo".to_string(),
        };
        tx.create_commit(params).unwrap();
        tx.commit().unwrap();

        remove_db("data/test_create_commit");
    }

    #[test]
    fn test_create_commit_twice() {
        let db = Database::new_rocksdb("data/test_create_commit_twice").unwrap();
        let tx = db.transaction();
        let params = CreateCommitParams {
            commit: &"1234567890abcdef".to_string(),
            server: &"github.com".to_string(),
            owner: &"owner".to_string(),
            repo: &"repo".to_string(),
        };
        tx.create_commit(params.clone()).unwrap();
        let err = tx.create_commit(params.clone()).unwrap_err();
        assert_eq!(err.kind(), Some(ErrorKind::CommitExists));

        remove_db("data/test_create_commit_twice");
    }

    #[test]
    fn test_create_artifact() {
        let db = Database::new_rocksdb("data/test_create_artifact").unwrap();
        let tx = db.transaction();
        let params = CreateArtifactParams {
            time: &1234567890,
            commit: &"1234567890abcdef".to_string(),
            path: &"path/to/artifact".to_string(),
        };
        tx.create_artifact(params).unwrap();
        tx.commit().unwrap();

        remove_db("data/test_create_artifact");
    }

    #[test]
    fn test_create_artifact_twice() {
        let db = Database::new_rocksdb("data/test_create_artifact_twice").unwrap();
        let tx = db.transaction();
        let params = CreateArtifactParams {
            time: &1234567890,
            commit: &"1234567890abcdef".to_string(),
            path: &"path/to/artifact".to_string(),
        };
        tx.create_artifact(params.clone()).unwrap();
        let err = tx.create_artifact(params.clone()).unwrap_err();
        assert_eq!(err.kind(), Some(ErrorKind::ArtifactExists));

        remove_db("data/test_create_artifact_twice");
    }
}
