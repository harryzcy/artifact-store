use rocksdb;

const ROCKSDB_PATH: &str = "data/rocksdb";

pub enum Database {
    RocksDB(rocksdb::TransactionDB),
}

impl Database {
    pub fn new_rocksdb() -> Result<Self, rocksdb::Error> {
        let db = rocksdb::TransactionDB::open_default(ROCKSDB_PATH)?;
        Ok(Database::RocksDB(db))
    }

    #[allow(dead_code)]
    pub fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, rocksdb::Error> {
        match self {
            Database::RocksDB(db) => db.get(key),
        }
    }

    #[allow(dead_code)]
    pub fn put(&self, key: &[u8], value: &[u8]) -> Result<(), rocksdb::Error> {
        match self {
            Database::RocksDB(db) => db.put(key, value),
        }
    }

    #[allow(dead_code)]
    pub fn delete(&self, key: &[u8]) -> Result<(), rocksdb::Error> {
        match self {
            Database::RocksDB(db) => db.delete(key),
        }
    }

    #[allow(dead_code)]
    pub fn transaction(&self) -> Transaction<'_> {
        match self {
            Database::RocksDB(db) => Transaction::RocksDB(db.transaction()),
        }
    }
}

pub enum Transaction<'db> {
    RocksDB(rocksdb::Transaction<'db, rocksdb::TransactionDB>),
}
