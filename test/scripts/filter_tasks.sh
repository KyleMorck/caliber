#!/usr/bin/env bash
set -euo pipefail

TEST_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
REPO_ROOT=$(cd "$TEST_DIR/.." && pwd)
FIXTURES_DIR="$TEST_DIR/fixtures"
TMP_ROOT="$TEST_DIR/.tmp"

mkdir -p "$TMP_ROOT"
RUN_DIR=$(mktemp -d "$TMP_ROOT/run-XXXXXX")
trap 'rm -rf "$RUN_DIR"' EXIT

cp -R "$FIXTURES_DIR/config" "$RUN_DIR/config"
cp -R "$FIXTURES_DIR/repos" "$RUN_DIR/repos"

if [ ! -d "$RUN_DIR/repos/project-01/.git" ]; then
  git -C "$RUN_DIR/repos/project-01" init -q
fi

export XDG_CONFIG_HOME="$RUN_DIR/config"

cargo build --manifest-path "$REPO_ROOT/Cargo.toml" --release --quiet

send_keys() {
  local keys="$1"
  echo "{\"type\": \"sendKeys\", \"keys\": $keys}"
}

snapshot() {
  echo '{"type": "takeSnapshot"}'
}

sleep_cmd() {
  local ms="$1"
  echo "{\"type\": \"sleep\", \"millis\": $ms}"
}

{
  sleep 0.5

  # Step 1: Ensure normal mode, switch to hub
  send_keys '["Escape", "Escape"]'
  sleep 0.2
  send_keys '["`"]'
  sleep 0.3

  # Step 2: Apply !tasks filter
  send_keys '["/", "!", "t", "a", "s", "k", "s", "Enter"]'
  sleep 0.3

  # Snapshot 1: Hub tasks filter
  snapshot
  sleep 0.2

  # Step 3: Exit filter, switch to project
  send_keys '["Escape"]'
  sleep 0.2
  send_keys '["`"]'
  sleep 0.3

  # Step 4: Apply !tasks #feature filter
  send_keys '["/", "!", "t", "a", "s", "k", "s", " ", "#", "f", "e", "a", "t", "u", "r", "e", "Enter"]'
  sleep 0.3

  # Snapshot 2: Project tasks filter
  snapshot
  sleep 0.2

  # Cleanup - quit the app
  send_keys '["Escape", ":", "q", "Enter"]'
  sleep 0.3

} | ht --size 120x40 --subscribe snapshot -- bash -c "export XDG_CONFIG_HOME='$RUN_DIR/config' && cd '$RUN_DIR/repos/project-01' && '$REPO_ROOT/target/release/caliber'" 2>&1

echo ""
echo "Test complete."
