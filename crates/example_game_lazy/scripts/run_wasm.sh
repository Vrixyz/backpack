#!/bin/bash

set -x
set -eo pipefail

static-web-server --root ./generated_wasm -w scripts/static-web-server.toml