pipebuilder apps
### Run Apps
manifest
```sh
RUST_LOG=info PIPEBUILDER_CONFIG_FILE=resources/manifest.yml cargo run --bin manifest
```
builder
```sh
RUST_LOG=info PIPEBUILDER_CONFIG_FILE=resources/builder.yml cargo run --bin builder
```
scheduler
```sh
RUST_LOG=info PIPEBUILDER_CONFIG_FILE=resources/scheduler.yml cargo run --bin scheduler
```
api
```sh
RUST_LOG=info PIPEBUILDER_CONFIG_FILE=resources/api.yml cargo run --bin api
```
### Endpoints
health
```sh
grpcurl -plaintext -import-path ../pipebuilder_common/proto -proto health.proto 127.0.0.1:19000 health.Health/Health
```