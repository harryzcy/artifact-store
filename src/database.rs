use rocksdb;

const DB_PATH: &str = "data/rocksdb";

pub fn init_db() -> Result<rocksdb::TransactionDB, rocksdb::Error> {
    let db = rocksdb::TransactionDB::open_default(DB_PATH)?;
    Ok(db)
}
