# Database Design

All database keys starts with a constant string defining its namespace. There are currently four different namespaces, `repo`, `commit`, `commit_time`, `artifact`, used for powering four different kind of APIs.

Key components are joined by `#`. Any `#` or `\` inside a component is escaped with a leading `\`, so the separator stays unambiguous. The one exception is a trailing fixed-width binary component, which is written verbatim: its width is known up front, so it needs no escaping, and inserting escape bytes would shift the byte-wise comparison RocksDB uses to order keys.

## `repo`

It's used for querying all repositories stored.

Key: `repo#{server}#{owner}#{repo}`
Value:
    - time_added: the timestamp since epoch

## `commit`

It's used for storing all commits in a repository ordered by commit hash.

Key: `commit#{server}#{owner}#{repo}#{commit}`
Value:
    - time_added: the timestamp since epoch

## `commit_time`

It's storing all commits ordered by the timestamp that commit is added.

Key: `commit_time#{server}#{owner}#{repo}#{time}`, where `{time}` is the 16-byte big-endian `u128` nanosecond timestamp, stored unescaped so that RocksDB's byte order matches chronological order
Value:
    - commit: commit hash

## `artifact`

It's storing all artifacts grouped by the commit hash.

Key: `artifact#{commit}#{path}`
Value:
    - time_added: the timestamp since epoch

Because in `artifact` namespace, path is grouped by commit hash, it's expected that commit hashes are unique among all repositories. Since Git now uses SHA256 as the hash function (replacing old SHA1 based hash prior to 2018), the condition is satisfied unless SHA256 is vulnerable to collision attacks sometime in the future, which is not likely.
