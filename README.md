<div align="center">
<img src=".github/assets/banner.png"></img>

[![Build Status]][travis]

[Build Status]: https://github.com/pipebase/pipebuilder/actions/workflows/ci.yml/badge.svg
[travis]: https://github.com/pipebase/pipebuilder/actions?branch%3Amain

</div>
<br />

`pipebuilder` is a CI for [`pipebase`] apps

## Overview
`pipebulder` is composed of five main components
* **api**: exposes the `pipebuilder` restful api.
* **builder**: build and publish [`pipebase`] app given manifest.
* **repository**: store app manifests and binaries
* **scheduler**: watch builders and assign build request.
* **pbctl**: command-line tool allows you to run commands against `pipebuilder` api

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
setup CI
```sh
docker-compose -f examples/docker-compose.yml up -d
```
create namespace
```sh
pbctl create namespace -i dev
```
create project in namespace
```sh
pbctl create project -n dev -i timer
```
push app manifest
```sh
pbctl push manifest -n dev -i timer -f examples/timer/pipe.yml
```
build app
```sh
pbctl create build -n dev -i timer -v 0 -t x86_64-apple-darwin
```
check build status till build succeed
```sh
pbctl list build -n dev
```
pull app binary and run
```
cd examples/timer && \
pbctl pull app -n dev -i timer -v 0 && \
chmod +x app && \
./app
```

[`cargo`]: https://doc.rust-lang.org/cargo/
[`pipebase`]: https://github.com/pipebase/pipebase/tree/main/pipebase