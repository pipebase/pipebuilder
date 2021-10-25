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
