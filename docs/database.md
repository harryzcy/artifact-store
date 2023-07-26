# Database Schema Definition

## Tables

| Table Name | Description |
| ---------- | ----------- |
| `system` | system info |
| `commits` | commits of all repos |
| `artifacts` | artifact info |

## Table Schemas

1. `system`

The `system` table contains database version information. It should only contain one row that stores the version integer.

```sql
CREATE TABLE IF NOT EXISTS system (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  version INTEGER NOT NULL,
)
```

This information currently is not used, but will be used in the future for database migrations. After each migration, the version number will by incremented by one.

1. `commits`

The `commit` table stores all the commits from all repositories.

```sql
CREATE TABLE IF NOT EXISTS commits (
    sha TEXT NOT NULL PRIMARY KEY,
    server TEXT NOT NULL,
    owner TEXT NOT NULL,
    repo TEXT NOT NULL,
    created_at TEXT NOT NULL
)
```

The table uses commit hash as the primary key `sha`, so it depends on the assumption that commit hash is unique across all repositories, which is highly likely to be true. As *artifact-store* is used for storing artifacts for new commits, and as Git use SHA-256 now, the hash will be unique unless a collision attach for SHA-256 is discovered.

The table additionally includes the information associated with commits. The column `created_at` is the time that the new commit is added to the database, not when commit is created or when Git server receives the commit. Due to this fact, it's important that CI jobs report the commits in the correct order.

1. `artifacts`

The `artifacts` table stores the artefact information and its associated commit.

```sql
CREATE TABLE IF NOT EXISTS artifacts (
    commit_sha TEXT NOT NULL,
    path TEXT NOT NULL,
    hash TEXT NOT NULL,
    hash_type TEXT NOT NULL,
    created_at TEXT NOT NULL,
    UNIQUE (commit_hash, path)
)
```
