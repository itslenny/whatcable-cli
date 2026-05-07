#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

cargo fmt --check

cargo clippy -- -D warnings

"$SCRIPT_DIR/build-release.sh"

cargo test