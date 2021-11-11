<div align="center">
<img src=".github/assets/banner.png"></img>

[![Build Status]][travis]

[Build Status]: https://github.com/pipebase/pipebuilder/actions/workflows/ci.yml/badge.svg
[travis]: https://github.com/pipebase/pipebuilder/actions?branch%3Amain

</div>
<br />

`pipebuilder` is a CI for [`pipebase`] apps

[Examples] | [Development]

## Overview
`pipebulder` is composed of five main components
* **api**: exposes the `pipebuilder` restful api.
* **builder**: build and publish [`pipebase`] app given manifest.
* **repository**: store app manifests and binaries
* **scheduler**: watch builders and assign build request.
* **pbctl**: command-line tool allows you to run commands against `pipebuilder` api

[`pipebase`]: https://github.com/pipebase/pipebase/tree/main/pipebase
[Examples]: https://github.com/pipebase/pipebuider/tree/main/examples
[Development]: https://github.com/pipebase/pipebuider/tree/main/e2e