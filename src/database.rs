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

    #[allow(dead_code)]
    pub fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, rocksdb::Error> {
        match self {
            Database::RocksDB(db) => db.get(key),
            Database::MockDB => Ok(None),
        }
    }

    #[allow(dead_code)]
    pub fn put(&self, key: &[u8], value: &[u8]) -> Result<(), rocksdb::Error> {
        match self {
            Database::RocksDB(db) => db.put(key, value),
            Database::MockDB => Ok(()),
        }
    }

    #[allow(dead_code)]
    pub fn delete(&self, key: &[u8]) -> Result<(), rocksdb::Error> {
        match self {
            Database::RocksDB(db) => db.delete(key),
            Database::MockDB => Ok(()),
        }
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
    pub fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, rocksdb::Error> {
        match self {
            Transaction::RocksDB(tx) => tx.get(key),
        }
    }

    pub fn put(&self, key: &[u8], value: &[u8]) -> Result<(), rocksdb::Error> {
        match self {
            Transaction::RocksDB(tx) => tx.put(key, value),
        }
    }

    pub fn delete(&self, key: &[u8]) -> Result<(), rocksdb::Error> {
        match self {
            Transaction::RocksDB(tx) => tx.delete(key),
        }
    }

    pub fn commit(self) -> Result<(), rocksdb::Error> {
        match self {
            Transaction::RocksDB(tx) => tx.commit(),
        }
    }
}
