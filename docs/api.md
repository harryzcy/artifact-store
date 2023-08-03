# API Design

Currently, Artifact Store exposes a set of REST APIs.

## Index

Method: `GET`

Endpoint: `/`

Response: name of the project.

## Ping

Method: `GET`

Endpoint: `/ping`

Response: `"pong"`

## List Repositories

Method: `GET`

Endpoint: `/repositories`

Response:

```json
{
  "repos": [
    {
      "server": "git.example.com",
      "owner": "username",
      "repo": "repository-name",
      "timeAdded": "RFC3339 string"
    }
  ]
}
```

## List Commits for a Repository

Method: `GET`

Endpoint: `/:server/:owner/:repo`

Response:

```json
{
  "server": "git.example.com",
  "owner": "username",
  "repo": "repository-name",
  "commits": [
    {
      "commit": "commit-hash",
      "timeAdded": "RFC3339 string"
    }
  ]
}
```

## List Artifacts for a Commit

Method: `GET`

Endpoint: `/:server/:owner/:repo/:commit`

Response:

```json
{
  "server": "git.example.com",
  "owner": "username",
  "repo": "repository-name",
  "commit": "commit-hash",
  "artifacts": [
    {
      "path": "artifact-path",
      "timeAdded": "RFC3339 string"
    }
  ]
}
```

## Upload Artifact

Method: `PUT`

Endpoint: `/:server/:owner/:repo/:commit/*path`

Response:

```json
{
  "code": 200,
  "message": "OK"
}
```

## Download Artifact

Method: `GET`

Endpoint: `/:server/:owner/:repo/:commit/*path`

Response: binary file
