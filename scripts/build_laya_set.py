#!/usr/bin/env python3
"""Emit System One JSONL from the train file `vikett train-set` wrote.

That file is the full train_rows set: paraphrases, catalogue goldens
(including extras), and exact suite cases. One JSONL line per train row.
The line count must match the train file. Does not train and does not
download weights. Run from the repo root. Snap id `live` is refused.
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from pathlib import Path


def load_rows(path: Path) -> list[dict]:
    data = json.loads(path.read_text())
    if not isinstance(data, list):
        raise SystemExit(f"{path} is not a row list")
    return data


def guest_snaps(path: Path) -> set[str]:
    snaps = json.loads(path.read_text())
    return {s["id"] for s in snaps if s.get("guest")}


def private_pages(path: Path) -> set[str]:
    pages = json.loads(path.read_text())
    return {p["id"] for p in pages if p.get("policy") == "private"}


def choice_for(row: dict, guests: set[str], private: set[str]) -> str:
    expect = row.get("expectPage")
    if row.get("snap") in guests and expect in private:
        return "none"
    if not expect:
        return "none"
    return expect


def criteria_probe(vikett: str, utterance: str, snap: str) -> dict:
    raw = subprocess.check_output(
        [vikett, "criteria", "-u", utterance, "-s", snap],
        text=True,
    )
    return json.loads(raw)


def main() -> None:
    parser = argparse.ArgumentParser(description="Build a Laya JSONL from the train file")
    parser.add_argument("--train", type=Path, required=True)
    parser.add_argument("--snaps", type=Path, default=Path("ontology/snaps.json"))
    parser.add_argument("--pages", type=Path, default=Path("ontology/pages.json"))
    parser.add_argument("--vikett", default="vikett")
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()

    rows = load_rows(args.train)
    for row in rows:
        if row.get("snap") == "live":
            raise SystemExit("snap id live is refused")
        if not row.get("id") or "utterance" not in row:
            raise SystemExit("train row missing id or utterance")

    guests = guest_snaps(args.snaps)
    private = private_pages(args.pages)
    args.out.parent.mkdir(parents=True, exist_ok=True)
    written = 0
    with args.out.open("w", encoding="utf-8") as fh:
        for row in rows:
            probe = criteria_probe(args.vikett, row["utterance"], row["snap"])
            criteria = probe.get("criteria") or {}
            for text in criteria.values():
                if "hyprctl" in text or "wpctl" in text:
                    raise SystemExit("criteria contain a walk string")
            body = {
                "state": probe.get("state"),
                "questions": probe.get("questions"),
                "answer": {"page": {"choice": choice_for(row, guests, private)}},
            }
            fh.write(json.dumps(body, ensure_ascii=False) + "\n")
            written += 1
    # JSONL line count must equal the train file. Do not drop rows.
    if written != len(rows):
        raise SystemExit(f"jsonl rows {written} != train file {len(rows)}")
    print(f"wrote {written} rows to {args.out}", file=sys.stderr)


if __name__ == "__main__":
    main()
