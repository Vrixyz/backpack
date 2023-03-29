#!/bin/bash

set -x
set -eo pipefail

static-web-server --port 8787 --root ./generated_wasm