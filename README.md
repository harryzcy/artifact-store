# Artifact Store

The missing piece in my build & deployment lifecycle. It stores the artifacts from by CI pipeline.

## Quick Start

The recommended way to run is to use Docker:

```shell
chown -R 65532:65532 /PATH/TO/DATA
docker run -v /PATH/TO/DATA:/data -p 3001:3001 ghcr.io/harryzcy/artifact-store
```

Note: the docker image uses `nonroot` user (UID and GID: 65532) by default,
so when mounting persistent volume, the permission need to be set accordingly. For more details, please refer to [docs/deployment.md](docs/deployment.md).

## Runtime Environment Variables

- `DATA_PATH`: the directory to store all the data, default to `/data`
- `ROCKSDB_PATH`: the path for RocksDB, default to `${DATA_PATH}/rocksdb`
- `ARTIFACTS_PATH`: the path to store artifact files, default to `${DATA_PATH}/artifacts`

## API

Please refer to [API docs](docs/api.md)

## Database Design

Please refer to [Database docs](docs/database.md)
