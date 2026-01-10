#!/usr/bin/env bash
set -euo pipefail

TEST_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
REPO_ROOT=$(cd "$TEST_DIR/.." && pwd)
FIXTURES_DIR="$TEST_DIR/fixtures"
TMP_ROOT="$TEST_DIR/.tmp"

mkdir -p "$TMP_ROOT"
RUN_DIR=$(mktemp -d "$TMP_ROOT/run-XXXXXX")
trap 'rm -rf "$RUN_DIR"' EXIT

cp -R "$FIXTURES_DIR/config" "$RUN_DIR/config"
cp -R "$FIXTURES_DIR/repos" "$RUN_DIR/repos"

if [ -d "$RUN_DIR/repos/project-01" ] && [ ! -d "$RUN_DIR/repos/project-01/.git" ]; then
  git -C "$RUN_DIR/repos/project-01" init -q
fi

echo "Fixtures prepared at: $RUN_DIR"
echo "Config dir: $RUN_DIR/config"
echo "Project dir: $RUN_DIR/repos/project-01"

export XDG_CONFIG_HOME="$RUN_DIR/config"
cd "$RUN_DIR/repos/project-01"

cargo run --manifest-path "$REPO_ROOT/Cargo.toml" --release
