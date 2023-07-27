use rocksdb;

const ROCKSDB_PATH: &str = "data/rocksdb";

#[allow(dead_code)]
pub enum Database {
    RocksDB(rocksdb::TransactionDB),
    MockDB,
}

impl Database {
    pub fn new_rocksdb() -> Result<Self, rocksdb::Error> {
        let db = rocksdb::TransactionDB::open_default(ROCKSDB_PATH)?;
        Ok(Database::RocksDB(db))
    }

    #[allow(dead_code)]
    pub fn new_mockdb() -> Self {
        Database::MockDB
    }

    pub fn transaction(&self) -> Transaction<'_> {
        match self {
            Database::RocksDB(db) => Transaction::RocksDB(db.transaction()),
            Database::MockDB => panic!("not implemented"),
        }
    }
}

pub enum Transaction<'db> {
    RocksDB(rocksdb::Transaction<'db, rocksdb::TransactionDB>),
}

impl Transaction<'_> {
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

    pub fn rollback(&self) -> Result<(), rocksdb::Error> {
        match self {
            Transaction::RocksDB(tx) => tx.rollback(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorKind {
    CommitExists,
    ArtifactExists,
}

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

pub struct CreateCommitParams<'a> {
    pub commit: &'a str,
    pub server: &'a str,
    pub owner: &'a str,
    pub repo: &'a str,
}

pub struct CreateArtifactParams<'a> {
    pub time: &'a u128,
    pub commit: &'a str,
    pub path: &'a str,
}
