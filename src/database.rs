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
    pub fn create_commit(&self, params: CreateCommitParams) -> Result<(), rocksdb::Error> {
        let commit_key = format!("commit:{}", params.commit);
        let commit_value = format!("{}/{}/{}", params.server, params.owner, params.repo);

        let repo_key = format!("repo:{}/{}", commit_value, params.commit);
        let repo_value = params.commit;

        match self {
            Transaction::RocksDB(tx) => {
                let commit_key_bytes = commit_key.as_bytes();
                let exists = tx.get(commit_key_bytes)?.is_some();
                if exists {
                    return Ok(());
                }
                tx.put(commit_key_bytes, commit_value.as_bytes())?;
                tx.put(repo_key.as_bytes(), repo_value.as_bytes())?;
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

pub struct CreateCommitParams {
    pub commit: String,
    pub server: String,
    pub owner: String,
    pub repo: String,
}
