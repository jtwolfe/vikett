#!/usr/bin/env bash
# Extensive prompt suite: lexical is the gate; Laya is compared if :8009 is up.
# VIKETT_LAYA_GATE=1 (unset by default) fails a down server, a holdout wrong
# walk, or holdout take-match under 85%. It does not mark any page traversable.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
cargo build --release -q
BIN=./target/release/vikett

echo "======== lexical must-pass ========"
"$BIN" suite --referee lexical

laya_listening() {
  curl -fsS -o /dev/null --max-time 2 http://127.0.0.1:8009/health 2>/dev/null
}

# Catalog holdouts only (`--tag holdout`), not goldens. An empty tag makes
# the binary exit non-zero ("holdout tag matched nothing"). Take-match is
# page id + required slots. Any wrong_act fails, which is stricter than the
# under-1% false-walk bar. This does not publish an accuracy.
holdout_gate() {
  local tmp status py
  tmp="$(mktemp)"
  set +e
  "$BIN" suite --referee laya --tag holdout --format json >"$tmp"
  status=$?
  python3 - "$tmp" "$status" <<'PY'
import json
import sys

path, status = sys.argv[1], int(sys.argv[2])
raw = open(path, encoding="utf-8").read().strip()
if not raw:
    sys.exit(status or 1)
try:
    doc = json.loads(raw)
except json.JSONDecodeError:
    sys.stderr.write(raw + "\n")
    sys.exit(status or 1)
if doc.get("skipped"):
    sys.exit("Laya skipped — VIKETT_LAYA_GATE=1 requires the server")
total = int(doc["total"])
passed = int(doc["pass"])
wrong = int(doc["wrong_acts"])
results = doc.get("results") or []
if total <= 0 or not results:
    sys.exit("holdout tag matched nothing")
for row in results:
    tags = row.get("tags") or []
    if "holdout" not in tags or "golden" in tags:
        sys.exit(f"{row.get('id')} is not a Catalog holdout case")
print(
    f"holdout take-match {passed}/{total}  wrong_act={wrong}  "
    "(need take-match >= 85%, wrong_act = 0; false-walk bar is under 1%)"
)
failed = False
if wrong > 0:
    print(f"holdout wrong_act {wrong} > 0", file=sys.stderr)
    failed = True
if passed * 100 < total * 85:
    print(f"holdout take-match {passed}/{total} is under 85%", file=sys.stderr)
    failed = True
if failed:
    for row in results:
        if row.get("pass"):
            continue
        mark = "ACT" if row.get("wrong_act") else "miss"
        print(
            f"  {mark} {row.get('id')} {row.get('utterance')!r} "
            f"expected {row.get('expected')} got {row.get('got')}",
            file=sys.stderr,
        )
    sys.exit(1)
if status != 0:
    sys.exit(status)
PY
  py=$?
  set -e
  rm -f "$tmp"
  if [[ "$py" -ne 0 ]]; then
    exit "$py"
  fi
}

if [[ "${VIKETT_LAYA_GATE:-}" == "1" ]]; then
  echo
  echo "======== laya holdout gate ========"
  if ! laya_listening; then
    echo "Laya server not on :8009 — VIKETT_LAYA_GATE=1 requires it." >&2
    echo "  edgejev serve --model ./jev-int8 --port 8009" >&2
    exit 1
  fi
  holdout_gate
  echo
  echo "======== laya compare (miss-to-silence is non-strict; wrong_act fails) ========"
  "$BIN" suite --referee laya
else
  if laya_listening; then
    echo
    echo "======== laya compare (miss-to-silence is non-strict; wrong_act fails) ========"
    "$BIN" suite --referee laya
  else
    echo
    echo "Laya server not on :8009 — lexical only. Start with:"
    echo "  edgejev serve --model ./jev-int8 --port 8009"
  fi
fi
