Examples for pipebuilder demo

## Install Tools
install [`cargo`]
```sh
curl https://sh.rustup.rs -sSf | sh
```
install `pbctl`
```sh
cargo install pipebuilder --bin pbctl
```
## Quick Start
clone git repository and setup CI
```sh
docker-compose -f examples/ci.yml up -d
```
create namespace, project, manifest
```sh
pbctl create namespace -i dev && \
pbctl create project -n dev -i timer && \
pbctl push manifest -n dev -i timer -f examples/timer/pipe.yml
```
trigger build
```sh
pbctl create build -n dev -i timer -v 0 -t x86_64-unknown-linux-gnu
```
wait for build succeed
```sh
pbctl list build -n dev
```
download and run application
```
cd examples/timer && \
pbctl pull app -n dev -i timer -v 0 && \
chmod +x app && \
./app
```

[`cargo`]: https://doc.rust-lang.org/cargo/