# Artifact Store

The missing piece in my build & deployment lifecycle. It stores the artifacts from by CI pipeline.

## Runtime Environment Variables

- `DATA_PATH`: the directory to store all the data, default to `/data`
- `ROCKSDB_PATH`: the path for RocksDB, default to `${DATA_PATH}/rocksdb`
- `ARTIFACTS_PATH`: the path to store artifact files, default to `${DATA_PATH}/artifacts`

## API

Please refer to [API docs](docs/api.md)

## Database Design

Please refer to [Database docs](docs/database.md)
