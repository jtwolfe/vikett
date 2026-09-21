#!/usr/bin/env bash
# Extensive prompt suite: lexical is the gate; Laya is compared if :8009 is up.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
cargo build --release -q
BIN=./target/release/vikett

echo "======== lexical must-pass ========"
"$BIN" suite --referee lexical

if curl -fsS -o /dev/null --max-time 2 http://127.0.0.1:8009/health 2>/dev/null; then
  echo
  echo "======== laya compare (miss-to-silence is non-strict; wrong_act fails) ========"
  "$BIN" suite --referee laya
else
  echo
  echo "Laya server not on :8009 — lexical only. Start with:"
  echo "  edgejev serve --model ./jev-int8 --port 8009"
fi
