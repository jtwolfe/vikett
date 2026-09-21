#!/usr/bin/env bash
# Build a local EdgeJev INT8 bundle for Vikett's Laya referee.
#
# EdgeJev 0.3.2's [build] extra does not declare `onnxscript`, which Torch 2.14+
# requires for `torch.onnx.export(..., dynamo=True)`. This wrapper installs it.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OUT="${1:-$ROOT/jev-int8}"
PORT="${VIKETT_LAYA_PORT:-8009}"

if ! command -v uv >/dev/null; then
  echo "uv is required (https://docs.astral.sh/uv/)" >&2
  exit 1
fi

echo "==> edgejev[build] + onnxscript (Torch 2.14 dynamo exporter)"
uv tool install --force "edgejev[build]" --with onnxscript

EJ_PY="$(dirname "$(command -v edgejev)")/python"
if [[ ! -x "$EJ_PY" ]]; then
  EJ_PY="$HOME/.local/share/uv/tools/edgejev/bin/python"
fi
"$EJ_PY" -c "from torch.onnx._internal.exporter import _compat; import onnxscript; print('onnxscript', onnxscript.__version__, 'ok')"

echo "==> edgejev build --backend laya --out $OUT"
mkdir -p "$OUT"
edgejev build --backend laya --out "$OUT"

if [[ ! -f "$OUT/edgejev.json" ]]; then
  echo "build finished without $OUT/edgejev.json" >&2
  exit 1
fi

echo
echo "bundle ready:"
ls -lh "$OUT"
echo
echo "serve with:"
echo "  edgejev serve --model $OUT --port $PORT"
echo "then:"
echo "  cargo run --release -- tree -u 'a bit louder' -s desk --referee laya"
