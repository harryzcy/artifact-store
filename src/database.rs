use rocksdb;

pub fn init_db() -> Result<rocksdb::DB, rocksdb::Error> {
    let path = "data/rocksdb";

    let mut options = rocksdb::Options::default();
    options.create_if_missing(true);

    let db = rocksdb::DB::open(&options, path)?;

    Ok(db)
}
