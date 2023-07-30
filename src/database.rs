use serde::Serialize;
use time::OffsetDateTime;

type TransactionDB = rocksdb::OptimisticTransactionDB;

#[derive(Clone)]
pub struct GetRepoCommitsParams<'a> {
    pub server: &'a String,
    pub owner: &'a String,
    pub repo: &'a String,
}

#[derive(Clone, Serialize)]
pub struct CommitData {
    pub commit: String,
    #[serde(with = "time::serde::rfc3339")]
    pub time: OffsetDateTime,
}

#[derive(Clone)]
pub struct ExistsArtifactParams<'a> {
    pub server: &'a String,
    pub owner: &'a String,
    pub repo: &'a String,
    pub commit: &'a String,
    pub path: &'a String,
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
    pub commit: &'a String,
    pub path: &'a String,
}

#[derive(Clone)]
pub struct CreateRepositoryParams<'a> {
    pub server: &'a String,
    pub owner: &'a String,
    pub repo: &'a String,
}

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

    pub fn get_repo_commits(&self, params: GetRepoCommitsParams) -> Result<Vec<CommitData>, Error> {
        let key_start = format!(
            "commit_time#{}#{}#{}#{}",
            params.server, params.owner, params.repo, 0
        );
        let key_end = format!(
            "commit_time#{}#{}#{}#{}",
            params.server,
            params.owner,
            params.repo,
            u128::MAX
        );
        let mut commits = Vec::new();
        match self {
            Database::RocksDB(db) => {
                let mut iter = db.raw_iterator();
                iter.seek(key_start.as_bytes());
                while iter.valid() && iter.key().unwrap() < key_end.as_bytes() {
                    let raw_key = iter.key().unwrap();
                    let raw_value = iter.value().unwrap();
                    let key = std::str::from_utf8(raw_key).unwrap();
                    let value = std::str::from_utf8(raw_value).unwrap();
                    let mut parts = key.split('#');
                    parts.next(); // commit_time
                    parts.next(); // server
                    parts.next(); // owner
                    parts.next(); // repo
                    let time_millisecond = parts.next().unwrap().parse::<u128>().unwrap();
                    let time_seconds = (time_millisecond / 1000) as i64;
                    let time = OffsetDateTime::from_unix_timestamp(time_seconds).unwrap();
                    commits.push(CommitData {
                        commit: value.to_string(),
                        time,
                    });
                    iter.next();
                }
            }
        };

        Ok(commits)
    }

    pub fn exists_artifact(&self, params: ExistsArtifactParams) -> Result<bool, Error> {
        let commit_key = format!(
            "commit#{}#{}#{}#{}",
            params.server, params.owner, params.repo, params.commit
        );
        let exists = match self {
            Database::RocksDB(db) => db.get(commit_key.as_bytes())?.is_some(),
        };
        if !exists {
            return Ok(false);
        }

        let artifact_key = format!("artifact#{}#{}", params.commit, params.path);
        let exists = match self {
            Database::RocksDB(db) => db.get(artifact_key.as_bytes())?.is_some(),
        };
        Ok(exists)
    }
}

pub enum Transaction<'db> {
    RocksDB(rocksdb::Transaction<'db, TransactionDB>),
}

impl Transaction<'_> {
    /// Stores the repository data in the database
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
                tx.put(key_bytes, value)?;
            }
        }
        Ok(())
    }

    /// Store the commit data in the database.
    pub fn create_commit_if_not_exists(
        &self,
        time: u128,
        params: CreateCommitParams,
    ) -> Result<(), Error> {
        let commit_key = format!(
            "commit#{}#{}#{}#{}",
            params.server, params.owner, params.repo, params.commit
        );
        let commit_value = time.to_be_bytes();

        let commit_time_key = [
            format!(
                "commit_time#{}#{}#{}#",
                params.server, params.owner, params.repo
            )
            .as_bytes(),
            &time.to_be_bytes(),
        ]
        .concat();
        let commit_time_value = params.commit.as_bytes();

        match self {
            Transaction::RocksDB(tx) => {
                let commit_key_bytes = commit_key.as_bytes();
                let exists = tx.get(commit_key_bytes)?.is_some();
                if exists {
                    return Ok(());
                }
                tx.put(commit_key_bytes, commit_value)?;
                tx.put(commit_time_key, commit_time_value)?;
            }
        }
        Ok(())
    }

    /// Store the artifact data in the database.
    /// If the artifact already exists, return an error.
    pub fn create_artifact(&self, time: u128, params: CreateArtifactParams) -> Result<(), Error> {
        let key = format!("artifact#{}#{}", params.commit, params.path);

        match self {
            Transaction::RocksDB(tx) => {
                let exists = tx.get(&key)?.is_some();
                if exists {
                    return Err(Error::Generic(format!(
                        "artifact already exists: {}",
                        params.path
                    )));
                }

                tx.put(key, time.to_be_bytes())?;
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

#[derive(Debug)]
pub enum Error {
    RocksDB(rocksdb::Error),
    Generic(String),
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
        let time = 1234567890;
        let params = CreateCommitParams {
            commit: &"1234567890abcdef".to_string(),
            server: &"github.com".to_string(),
            owner: &"owner".to_string(),
            repo: &"repo".to_string(),
        };
        tx.create_commit_if_not_exists(time, params).unwrap();
        tx.commit().unwrap();

        remove_db("data/test_create_commit");
    }

    #[test]
    fn test_create_commit_twice() {
        let db = Database::new_rocksdb("data/test_create_commit_twice").unwrap();
        let tx = db.transaction();
        let time = 1234567890;
        let params = CreateCommitParams {
            commit: &"1234567890abcdef".to_string(),
            server: &"github.com".to_string(),
            owner: &"owner".to_string(),
            repo: &"repo".to_string(),
        };
        tx.create_commit_if_not_exists(time, params.clone())
            .unwrap();
        tx.create_commit_if_not_exists(time, params.clone())
            .unwrap();

        remove_db("data/test_create_commit_twice");
    }

    #[test]
    fn test_create_artifact() {
        let db = Database::new_rocksdb("data/test_create_artifact").unwrap();
        let tx = db.transaction();
        let time = 1234567890;
        let params = CreateArtifactParams {
            commit: &"1234567890abcdef".to_string(),
            path: &"path/to/artifact".to_string(),
        };
        tx.create_artifact(time, params).unwrap();
        tx.commit().unwrap();

        remove_db("data/test_create_artifact");
    }

    #[test]
    fn test_create_artifact_twice() {
        let db = Database::new_rocksdb("data/test_create_artifact_twice").unwrap();
        let tx = db.transaction();
        let time = 1234567890;
        let params = CreateArtifactParams {
            commit: &"1234567890abcdef".to_string(),
            path: &"path/to/artifact".to_string(),
        };
        tx.create_artifact(time, params.clone()).unwrap();
        let err = tx.create_artifact(time, params.clone()).unwrap_err();
        assert!(matches!(err, Error::Generic(_)));

        remove_db("data/test_create_artifact_twice");
    }
}
