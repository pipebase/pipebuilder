`pipebuilder_mock` is a [`pipebuilder`] mock server for testing

## Local Development
```sh
# at project root
RUST_LOG=info PIPEBUILDER_LOG_FORMATTER=full PIPEBUILDER_CONFIG_FILE=pipebuilder_mock/resources/mock.yml cargo run --bin mock
```

[`pipebuilder`]: https://github.com/pipebase/pipebuilder