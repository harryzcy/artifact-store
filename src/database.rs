use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

type TransactionDB = rocksdb::OptimisticTransactionDB;

const NANOSECONDS_PER_SECOND: i64 = 1_000_000_000;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RepoData {
    pub server: String,
    pub owner: String,
    pub repo: String,
    pub time_added: OffsetDateTime,
}

#[derive(Clone)]
pub struct ExistsCommitParams<'a> {
    pub server: &'a String,
    pub owner: &'a String,
    pub repo: &'a String,
    pub commit: &'a String,
}

#[derive(Clone)]
pub struct ListRepoCommitsParams<'a> {
    pub server: &'a String,
    pub owner: &'a String,
    pub repo: &'a String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommitData {
    pub commit: String,
    #[serde(with = "time::serde::rfc3339")]
    pub time_added: OffsetDateTime,
}

#[derive(Clone)]
pub struct GetLatestCommitParams<'a> {
    pub server: &'a String,
    pub owner: &'a String,
    pub repo: &'a String,
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
pub struct ListArtifactsParams<'a> {
    pub server: &'a String,
    pub owner: &'a String,
    pub repo: &'a String,
    pub commit: &'a String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactData {
    pub path: String,
    #[serde(with = "time::serde::rfc3339")]
    pub time_added: OffsetDateTime,
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

#[derive(Serialize, Deserialize)]
struct RepoValue {
    time_added: u128,
}

#[derive(Serialize, Deserialize)]
struct CommitValue {
    time_added: u128,
}

#[derive(Serialize, Deserialize)]
struct CommitTimeValue {
    commit: String,
}

#[derive(Serialize, Deserialize)]
struct ArtifactValue {
    time_added: u128,
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

    pub fn list_repos(&self) -> Result<Vec<RepoData>, Error> {
        let key_prefix = serialize_key(vec!["repo".as_bytes()]);
        self.get_by_prefix(
            key_prefix,
            |key, value| {
                // parts: ["repo", server, owner, repo]
                let key_parts = deserialize_key(key);
                // let server = std::str::from_utf8(&key_parts[1]).unwrap();
                let server = std::str::from_utf8(&key_parts[1]).unwrap();
                let owner = std::str::from_utf8(&key_parts[2]).unwrap();
                let repo = std::str::from_utf8(&key_parts[3]).unwrap();
                let value = serde_json::from_slice::<RepoValue>(value).unwrap();
                let time_seconds = value.time_added as i64 / NANOSECONDS_PER_SECOND;
                let time_added = OffsetDateTime::from_unix_timestamp(time_seconds).unwrap();
                Ok(RepoData {
                    server: server.to_string(),
                    owner: owner.to_string(),
                    repo: repo.to_string(),
                    time_added,
                })
            },
            None,
        )
    }

    pub fn exists_commit(&self, params: ExistsCommitParams) -> Result<bool, Error> {
        let commit_key = serialize_key(vec![
            "commit".as_bytes(),
            params.server.as_bytes(),
            params.owner.as_bytes(),
            params.repo.as_bytes(),
            params.commit.as_bytes(),
        ]);
        let exists = match self {
            Database::RocksDB(db) => db.get(commit_key)?.is_some(),
        };
        Ok(exists)
    }

    pub fn list_repo_commits(
        &self,
        params: ListRepoCommitsParams,
    ) -> Result<Vec<CommitData>, Error> {
        let key_prefix = serialize_key(vec![
            "commit_time".as_bytes(),
            params.server.as_bytes(),
            params.owner.as_bytes(),
            params.repo.as_bytes(),
        ]);

        self.get_by_prefix(
            key_prefix,
            |key, value| {
                // parts: ["commit_time", server, owner, repo, time]
                let key_parts = deserialize_key(key);
                let time_part = key_parts.last().unwrap();

                // time_part is expected to be a u128
                let time = extract_time(time_part);

                let value_str = std::str::from_utf8(value).unwrap();
                let value = serde_json::from_str::<CommitTimeValue>(value_str).unwrap();
                Ok(CommitData {
                    commit: value.commit,
                    time_added: time,
                })
            },
            Some(true),
        )
    }

    pub fn get_latest_commit(&self, params: GetLatestCommitParams) -> Result<String, Error> {
        let mut search_key = serialize_key(vec![
            "commit_time".as_bytes(),
            params.server.as_bytes(),
            params.owner.as_bytes(),
            params.repo.as_bytes(),
        ]);
        search_key.push(b'$');

        match self {
            Database::RocksDB(db) => {
                let mut iter = db.raw_iterator();
                iter.seek_for_prev(&search_key);
                if iter.valid() {
                    let value_raw = iter.value().unwrap();
                    let value_str = std::str::from_utf8(value_raw).unwrap();
                    let value = serde_json::from_str::<CommitTimeValue>(value_str).unwrap();
                    return Ok(value.commit);
                }
                Err(Error::Generic("no commits found".to_string()))
            }
        }
    }

    pub fn exists_artifact(&self, params: ExistsArtifactParams) -> Result<bool, Error> {
        let exists = self.exists_commit(ExistsCommitParams {
            server: params.server,
            owner: params.owner,
            repo: params.repo,
            commit: params.commit,
        })?;
        if !exists {
            return Ok(false);
        }

        let artifact_key = serialize_key(vec![
            "artifact".as_bytes(),
            params.commit.as_bytes(),
            params.path.as_bytes(),
        ]);
        let exists = match self {
            Database::RocksDB(db) => db.get(artifact_key)?.is_some(),
        };
        Ok(exists)
    }

    pub fn list_artifacts(&self, params: ListArtifactsParams) -> Result<Vec<ArtifactData>, Error> {
        let key_prefix = serialize_key(vec!["artifact".as_bytes(), params.commit.as_bytes()]);

        self.get_by_prefix(
            key_prefix,
            |key, value| {
                // parts: ["artifact", commit, path]
                let key_parts = deserialize_key(key);
                let path_raw = key_parts.last().unwrap();
                let path = std::str::from_utf8(path_raw).unwrap().to_string();

                let value_str = std::str::from_utf8(value).unwrap();
                let value = serde_json::from_str::<ArtifactValue>(value_str).unwrap();
                let time_seconds = value.time_added as i64 / NANOSECONDS_PER_SECOND;
                let time = OffsetDateTime::from_unix_timestamp(time_seconds).unwrap();

                Ok(ArtifactData {
                    path,
                    time_added: time,
                })
            },
            None,
        )
    }

    fn get_by_prefix<T>(
        &self,
        key_prefix: Vec<u8>,
        func: fn(key: &[u8], value: &[u8]) -> Result<T, Error>,
        reverse: Option<bool>,
    ) -> Result<Vec<T>, Error> {
        let mut key_start = key_prefix.clone();
        key_start.push(b'#');

        let mut key_end = key_prefix.clone();
        key_end.push(b'$');

        let mut result: Vec<T> = Vec::new();
        match self {
            Database::RocksDB(db) => {
                let mut iter = db.raw_iterator();
                let should_reverse = reverse.unwrap_or(false);
                if should_reverse {
                    iter.seek_for_prev(&key_end);
                } else {
                    iter.seek(&key_start);
                }
                while iter.valid() {
                    let raw_key = iter.key().unwrap();
                    if !raw_key.starts_with(&key_prefix) {
                        break;
                    }
                    let raw_value = iter.value().unwrap();
                    let value = func(raw_key, raw_value)?;
                    result.push(value);
                    if should_reverse {
                        iter.prev();
                    } else {
                        iter.next();
                    }
                }
                Ok(result)
            }
        }
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
        let key = serialize_key(vec![
            "repo".as_bytes(),
            params.server.as_bytes(),
            params.owner.as_bytes(),
            params.repo.as_bytes(),
        ]);
        let value = RepoValue { time_added: time };

        match self {
            Transaction::RocksDB(tx) => {
                let exists = tx.get(&key)?.is_some();
                if exists {
                    return Ok(());
                }
                tx.put(key, serde_json::to_string(&value).unwrap().as_bytes())?;
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
        let commit_key = serialize_key(vec![
            "commit".as_bytes(),
            params.server.as_bytes(),
            params.owner.as_bytes(),
            params.repo.as_bytes(),
            params.commit.as_bytes(),
        ]);
        let commit_value = CommitValue { time_added: time };

        let commit_time_key = serialize_key(vec![
            "commit_time".as_bytes(),
            params.server.as_bytes(),
            params.owner.as_bytes(),
            params.repo.as_bytes(),
            &time.to_be_bytes(),
        ]);
        let commit_time_value = CommitTimeValue {
            commit: params.commit.clone(),
        };

        match self {
            Transaction::RocksDB(tx) => {
                let exists = tx.get(&commit_key)?.is_some();
                if exists {
                    return Ok(());
                }
                tx.put(
                    commit_key,
                    serde_json::to_string(&commit_value).unwrap().as_bytes(),
                )?;
                tx.put(
                    commit_time_key,
                    serde_json::to_string(&commit_time_value)
                        .unwrap()
                        .as_bytes(),
                )?;
            }
        }
        Ok(())
    }

    /// Store the artifact data in the database.
    /// If the artifact already exists, return an error.
    pub fn create_artifact(&self, time: u128, params: CreateArtifactParams) -> Result<(), Error> {
        let key = serialize_key(vec![
            "artifact".as_bytes(),
            params.commit.as_bytes(),
            params.path.as_bytes(),
        ]);
        let value = ArtifactValue { time_added: time };

        match self {
            Transaction::RocksDB(tx) => {
                let exists = tx.get(&key)?.is_some();
                if exists {
                    return Err(Error::Generic(format!(
                        "artifact already exists: {}",
                        params.path
                    )));
                }

                tx.put(key, serde_json::to_string(&value).unwrap().as_bytes())?;
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

fn serialize_key(parts: Vec<&[u8]>) -> Vec<u8> {
    let mut result: Vec<u8> = Vec::new();
    for i in 0..parts.len() {
        let part = parts[i];
        for byte in part {
            if *byte == b'#' || *byte == b'\\' {
                result.push(b'\\');
            }
            result.push(*byte);
        }
        if i < parts.len() - 1 {
            result.push(b'#');
        }
    }
    result
}

fn deserialize_key(key: &[u8]) -> Vec<Vec<u8>> {
    let mut result: Vec<Vec<u8>> = Vec::new();
    let mut part: Vec<u8> = Vec::new();
    let mut escape = false;
    for byte in key {
        if escape {
            part.push(*byte);
            escape = false;
        } else if *byte == b'\\' {
            escape = true;
        } else if *byte == b'#' {
            result.push(part);
            part = Vec::new();
        } else {
            part.push(*byte);
        }
    }
    result.push(part);
    result
}

fn extract_time(bytes: &[u8]) -> OffsetDateTime {
    let time_nano = u128::from_be_bytes(bytes[0..16].try_into().unwrap());
    let time_seconds = time_nano as i64 / NANOSECONDS_PER_SECOND;
    OffsetDateTime::from_unix_timestamp(time_seconds).unwrap()
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    fn remove_db(path: &str) {
        let _ = std::fs::remove_dir_all(path);
    }

    #[test]
    fn test_serialize() {
        let key = vec![[0, 0, 0, 0, 0, 0, 0, 0, 23, 122, 45, 94, 28, 92, 20, 192].as_ref()];
        let serialized = serialize_key(key);
        assert_eq!(
            serialized,
            vec![0, 0, 0, 0, 0, 0, 0, 0, 23, 122, 45, 94, 28, 92, 92, 20, 192]
        );
    }

    #[test]
    fn test_deserialize() {
        let key = vec![0, 0, 0, 0, 0, 0, 0, 0, 23, 122, 45, 94, 28, 92, 92, 20, 192];
        let deserialized = deserialize_key(&key);
        assert_eq!(deserialized.len(), 1);
        assert_eq!(
            deserialized[0],
            vec![0, 0, 0, 0, 0, 0, 0, 0, 23, 122, 45, 94, 28, 92, 20, 192]
        );
    }

    #[test]
    fn test_key_simple() {
        let key = serialize_key(vec![
            "repo".as_bytes(),
            "github.com".as_bytes(),
            "owner".as_bytes(),
        ]);
        assert_eq!(key, "repo#github.com#owner".as_bytes());

        let deserialized = deserialize_key(&key);
        assert_eq!(deserialized.len(), 3);
        assert_eq!(deserialized[0], "repo".as_bytes());
        assert_eq!(deserialized[1], "github.com".as_bytes());
        assert_eq!(deserialized[2], "owner".as_bytes());
    }

    #[test]
    fn test_key_separator_escape() {
        let key = serialize_key(vec![
            "repo".as_bytes(),
            "github.com".as_bytes(),
            "owner#with#hashes".as_bytes(),
        ]);
        assert_eq!(key, "repo#github.com#owner\\#with\\#hashes".as_bytes());

        let deserialized = deserialize_key(&key);
        assert_eq!(deserialized.len(), 3);
        assert_eq!(deserialized[0], "repo".as_bytes());
        assert_eq!(deserialized[1], "github.com".as_bytes());
        assert_eq!(deserialized[2], "owner#with#hashes".as_bytes());
    }

    #[test]
    fn test_extract_time() {
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let time_bytes = time.to_be_bytes().to_vec();
        let extracted = extract_time(&time_bytes);
        assert_eq!(
            time as i64 / NANOSECONDS_PER_SECOND,
            extracted.unix_timestamp()
        );
    }

    #[test]
    fn test_extract_time_from_key() {
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        let bytes = serialize_key(vec![&time.to_be_bytes()]);
        let deserialized = deserialize_key(&bytes);
        assert_eq!(deserialized.len(), 1);
        assert_eq!(deserialized[0], time.to_be_bytes());

        let extracted = extract_time(deserialized.last().unwrap());
        assert_eq!(
            time as i64 / NANOSECONDS_PER_SECOND,
            extracted.unix_timestamp()
        );
    }

    #[test]
    fn test_list_repos() {
        let db = Database::new_rocksdb("data/test_list_repos").unwrap();
        let tx = db.transaction();
        let time = 1234567890;
        let params = CreateRepositoryParams {
            server: &"github.com".to_string(),
            owner: &"owner".to_string(),
            repo: &"repo".to_string(),
        };
        tx.create_repo_if_not_exists(time, params).unwrap();
        tx.commit().unwrap();

        let repos = db.list_repos().unwrap();
        assert_eq!(repos.len(), 1);
        assert_eq!(repos[0].server, "github.com");
        assert_eq!(repos[0].owner, "owner");
        assert_eq!(repos[0].repo, "repo");

        remove_db("data/test_list_repos");
    }

    #[test]
    fn test_list_repos_multiple() {
        let db = Database::new_rocksdb("data/test_list_repos_multiple").unwrap();
        let tx = db.transaction();
        let time = 1234567890;
        let params = CreateRepositoryParams {
            server: &"github.com".to_string(),
            owner: &"owner".to_string(),
            repo: &"repo".to_string(),
        };
        tx.create_repo_if_not_exists(time, params).unwrap();
        let params = CreateRepositoryParams {
            server: &"gitlab.com".to_string(),
            owner: &"owner".to_string(),
            repo: &"repo".to_string(),
        };
        tx.create_repo_if_not_exists(time, params).unwrap();
        let params = CreateRepositoryParams {
            server: &"gitlab.com".to_string(),
            owner: &"owner".to_string(),
            repo: &"repo-2".to_string(),
        };
        tx.create_repo_if_not_exists(time, params).unwrap();
        tx.commit().unwrap();

        let repos = db.list_repos().unwrap();
        assert_eq!(repos.len(), 3);
        assert_eq!(repos[0].server, "github.com");
        assert_eq!(repos[0].repo, "repo");
        assert_eq!(repos[1].server, "gitlab.com");
        assert_eq!(repos[1].repo, "repo");
        assert_eq!(repos[2].server, "gitlab.com");
        assert_eq!(repos[2].repo, "repo-2");

        remove_db("data/test_list_repos_multiple");
    }

    #[test]
    fn test_list_commits() {
        let db = Database::new_rocksdb("data/test_list_commits").unwrap();
        let tx = db.transaction();
        let time_nano = 1234567890 * NANOSECONDS_PER_SECOND as u128;
        let params = CreateCommitParams {
            commit: &"1234567890abcdef".to_string(),
            server: &"github.com".to_string(),
            owner: &"owner".to_string(),
            repo: &"repo".to_string(),
        };
        tx.create_commit_if_not_exists(time_nano, params).unwrap();
        tx.commit().unwrap();

        let commits = db
            .list_repo_commits(ListRepoCommitsParams {
                server: &"github.com".to_string(),
                owner: &"owner".to_string(),
                repo: &"repo".to_string(),
            })
            .unwrap();
        assert_eq!(commits.len(), 1);
        assert_eq!(commits[0].commit, "1234567890abcdef");
        assert_eq!(
            commits[0].time_added.unix_timestamp_nanos(),
            time_nano as i128
        );

        remove_db("data/test_list_commits");
    }

    #[test]
    fn test_list_commits_order() {
        let db = Database::new_rocksdb("data/test_list_commits_multiple").unwrap();
        let tx = db.transaction();
        let params = CreateCommitParams {
            commit: &"commit-1".to_string(),
            server: &"github.com".to_string(),
            owner: &"owner".to_string(),
            repo: &"repo".to_string(),
        };
        tx.create_commit_if_not_exists(1234567890, params).unwrap();
        let params = CreateCommitParams {
            commit: &"commit-2".to_string(),
            server: &"github.com".to_string(),
            owner: &"owner".to_string(),
            repo: &"repo".to_string(),
        };
        tx.create_commit_if_not_exists(1234567891, params).unwrap();
        tx.commit().unwrap();

        let commits = db
            .list_repo_commits(ListRepoCommitsParams {
                server: &"github.com".to_string(),
                owner: &"owner".to_string(),
                repo: &"repo".to_string(),
            })
            .unwrap();
        assert_eq!(commits.len(), 2);
        assert_eq!(commits[0].commit, "commit-2");
        assert_eq!(commits[1].commit, "commit-1");

        remove_db("data/test_list_commits_multiple");
    }

    #[test]
    fn test_get_latest_commit() {
        let db = Database::new_rocksdb("data/test_get_latest_commit").unwrap();
        let tx = db.transaction();
        let params = CreateCommitParams {
            commit: &"commit-1".to_string(),
            server: &"github.com".to_string(),
            owner: &"owner".to_string(),
            repo: &"repo".to_string(),
        };
        tx.create_commit_if_not_exists(1234567890, params).unwrap();
        let params = CreateCommitParams {
            commit: &"commit-2".to_string(),
            server: &"github.com".to_string(),
            owner: &"owner".to_string(),
            repo: &"repo".to_string(),
        };
        tx.create_commit_if_not_exists(1234567891, params).unwrap();
        tx.commit().unwrap();

        let commit = db
            .get_latest_commit(GetLatestCommitParams {
                server: &"github.com".to_string(),
                owner: &"owner".to_string(),
                repo: &"repo".to_string(),
            })
            .unwrap();
        assert_eq!(commit, "commit-2");

        remove_db("data/test_get_latest_commit");
    }

    #[test]
    fn test_list_artifacts() {
        let db = Database::new_rocksdb("data/test_list_artifacts").unwrap();
        let tx = db.transaction();
        let time_milliseconds = 1234567890 * NANOSECONDS_PER_SECOND as u128;
        let params = CreateArtifactParams {
            commit: &"1234567890abcdef".to_string(),
            path: &"path/to/artifact".to_string(),
        };
        tx.create_artifact(time_milliseconds, params).unwrap();
        tx.commit().unwrap();

        let artifacts = db
            .list_artifacts(ListArtifactsParams {
                server: &"github.com".to_string(),
                owner: &"owner".to_string(),
                repo: &"repo".to_string(),
                commit: &"1234567890abcdef".to_string(),
            })
            .unwrap();
        assert_eq!(artifacts.len(), 1);
        assert_eq!(artifacts[0].path, "path/to/artifact");
        assert_eq!(artifacts[0].time_added.unix_timestamp(), 1234567890);

        remove_db("data/test_list_artifacts");
    }

    #[test]
    fn test_list_artifacts_multiple() {
        let db = Database::new_rocksdb("data/test_list_artifacts_multiple").unwrap();
        let tx = db.transaction();
        let time_milliseconds = 1234567890 * 1000;
        let params1 = CreateArtifactParams {
            commit: &"commit-1".to_string(),
            path: &"path/to/artifact-1".to_string(),
        };
        tx.create_artifact(time_milliseconds, params1).unwrap();
        let params2 = CreateArtifactParams {
            commit: &"commit-1".to_string(),
            path: &"path/to/artifact-2".to_string(),
        };
        tx.create_artifact(time_milliseconds, params2).unwrap();
        let params3 = CreateArtifactParams {
            commit: &"commit-2".to_string(),
            path: &"path/to/artifact-3".to_string(),
        };
        tx.create_artifact(time_milliseconds, params3).unwrap();
        tx.commit().unwrap();

        let artifacts_commit_1 = db
            .list_artifacts(ListArtifactsParams {
                server: &"github.com".to_string(),
                owner: &"owner".to_string(),
                repo: &"repo".to_string(),
                commit: &"commit-1".to_string(),
            })
            .unwrap();
        assert_eq!(artifacts_commit_1.len(), 2);
        assert_eq!(artifacts_commit_1[0].path, "path/to/artifact-1");
        assert_eq!(artifacts_commit_1[1].path, "path/to/artifact-2");

        let artifacts_commit_2 = db
            .list_artifacts(ListArtifactsParams {
                server: &"github.com".to_string(),
                owner: &"owner".to_string(),
                repo: &"repo".to_string(),
                commit: &"commit-2".to_string(),
            })
            .unwrap();
        assert_eq!(artifacts_commit_2.len(), 1);
        assert_eq!(artifacts_commit_2[0].path, "path/to/artifact-3");

        remove_db("data/test_list_artifacts_multiple");
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
