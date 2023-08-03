# Database Design

All database keys starts with a constant string defining its namespace. There are currently four different namespaces, `repo`, `commit`, `commit_time`, `artifact`, used for powering four different kind of APIs.

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

Key: `commit_time#{server}#{owner}#{repo}#{time}`
Value:
    - commit: commit hash

## `artifact`

It's storing all artifacts grouped by the commit hash.

Key: `artifact#{commit}#{path}`
Value:
    - time_added: the timestamp since epoch

Because in `artifact` namespace, path is grouped by commit hash, it's expected that commit hashes are unique among all repositories. Since Git now uses SHA256 as the hash function (replacing old SHA1 based hash prior to 2018), the condition is satisfied unless SHA256 is vulnerable to collision attacks sometime in the future, which is not likely.
