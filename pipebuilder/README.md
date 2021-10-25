pipebuilder apps
### Rpc Endpoints
health
```sh
grpcurl -plaintext -import-path ../pipebuilder_common/proto -proto health.proto 127.0.0.1:19000 health.Health/Health
```