pipebuilder e2e test workspace
## Local Development
install `pbctl`
```sh
# at project root
cargo install --path pipebuilder --bin pbctl
```
cleanup local data volume
```sh
# at project root
./e2e/setup-data-volume.sh
```
setup etcd
```sh
# at project root
docker-compose -f e2e/etcd.yml up -d
```
run `repository`, `builder`, `scheduler`, `api` services
```sh
# at project root
RUST_LOG=info PIPEBUILDER_CONFIG_FILE=e2e/resources/SERVICE.yml cargo run --bin SERVICE
```
## Test Sample App
go to test directory
```sh
cd e2e/tests/A_TEST_PROJECT
```
create namespace, project, manifest
```sh
pbctl create namespace -i dev && \
pbctl create project -n dev -i A_TEST_PROJECT && \
pbctl push manifest -n dev -i A_TEST_PROJECT -f pipe.yml
```
trigger build
```sh
pbctl create build -n dev -i A_TEST_PROJECT -v MANIFEST_VERSION && \
```
wait for build succeed
```
pbctl list build -n dev
```
download and run application
```
cd tests/A_TEST_PROJECT && \
pbctl pull -n dev -i A_TEST_PROJECT -v BUILD_VERSION && \
chmod +x app && \
./app
```
## Cleanup
shutdown internal node
```sh
pbctl shutdown builder -i builder0 && \
pbctl shutdown builder -i scheduler0 && \
pbctl shutdown builder -i repository0 && \
```
shutdown api server
```sh
curl -X POST http://localhost:16000/admin/shutdown \
    -H 'Content-Type: application/json' \
    -d '{}'
```
shutdown etcd
```sh
docker-compose -f e2e/etcd.yml down
```
cleanup data directory
```sh
./e2e/setup-data-volume.sh
```
## Run Integration Tests
```sh
RUST_TEST_TASKS=1 cargo test --package e2e --features itest
```