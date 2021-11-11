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
docker-compose -f e2e/etcd.yml down
docker-compose -f e2e/etcd.yml up -d
```
run `repository`, `builder`, `scheduler`, `api` services
```sh
# at project root
RUST_LOG=info PIPEBUILDER_CONFIG_FILE=e2e/resources/SERVICE.yml cargo run --bin SERVICE
```
## Run test
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
