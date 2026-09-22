# Model wiring

Status: research notes, 2026-09. Numbers are **published by the projects named**, not Vikett benches. Re-measure on the target box before picking a binary.

## What the referee is

A **System 1 typed-decision model**: you hand it a **state** and a dict of **typed questions**; it returns calibrated answers in **one forward pass**. No tokens. No JSON to parse from a decoder.

Three primitives ([Laya](https://huggingface.co/convaiinnovations/laya), Jev-compatible):

| Type | Returns |
| --- | --- |
| `choice` | one key from a criteria dict + per-key probabilities |
| `score` | expected level on an ordered rubric |
| `noul` | P(true) ∈ [0,1] |

That is the whole interface Vikett needs. A tool-calling LLM is a different, slower, leakier shape.

## Checkpoints

Consolidated Hub repo: [`convaiinnovations/laya`](https://huggingface.co/convaiinnovations/laya). Apache 2.0. Official runtime: [`NandhaKishorM/laya`](https://github.com/NandhaKishorM/laya) (`pip install laya`, `agent.system_one` / `predict`).

| Checkpoint | Backbone | Params | Ctx | Best at | Typed-decisions acc |
| --- | --- | --- | --- | --- | --- |
| `convaiinnovations/laya` (root) | ModernBERT-large | 421M | 512 | English triage, guardrails | **0.362** zero-shot |
| `laya-multilingual` | mmBERT-base, 256k vocab | 322M | 1024 (up to 8k) | 100+ languages, ~2.2× faster | **0.342** zero-shot |
| `laya-typed-decisions` | ModernBERT-large | 421M | 1024 | four synthetic workflows | **0.766** |

The typed-decisions bench is 400 cases / 2,000 decisions (invoice, security, customer service, agent-trace). Fine-tune **beats** TypeSafe Jev 1.13.0 (0.727) and the teacher self-agreement ceiling (0.735). By primitive: `noul` 0.857, `choice` 0.733, `score` 0.723.

**The base checkpoints sit below majority class on that bench.** Convai is explicit: Laya is a fast base to **specialise**, not a ready-made zero-shot engine. Vikett must fine-tune on goldens + paraphrases. Do not ship multilingual zero-shot as the household referee and hope.

## Runtimes (where it actually runs)

Hyprland / Omarchy is **Linux**. `laya-mlx` is Apple-only. Do not pick it for the workstation daemon.

| Runtime | Device | Latency (published) | Footprint | Use |
| --- | --- | --- | --- | --- |
| Official PyTorch `laya` | T4 GPU | 32.8 ms P50 / 1 q; 72.3 ms / 10 q batched | GPU | If the box has a GPU and we already pull torch |
| Official PyTorch CPU | CPU | 200–500 ms (Laya’s own note) | large | Avoid as the daemon |
| **[EdgeJev](https://github.com/yzfly/edgejev)** ONNX INT8 | 4 vCPU Xeon Cascade Lake | **15.6 ms** / 1 q, 44.8 ms / 3 q | **324 MB** | **Default Linux CPU path** |
| EdgeJev FP32 | same | 32.1 ms / 1 q | 1290 MB | Bit-match upstream |
| [`@receptron/laya`](https://github.com/receptron/laya) | onnxruntime-node | ~140 ms / 3 q Apple CPU warm | ~1.7 GB FP32 download | Engine-in-TypeScript experiments |
| [`mizorewww/laya-mlx`](https://github.com/mizorewww/laya-mlx) | M3 Max Metal | 13.4 ms EN / 7.4 ms multilingual P50 | Apple | Laptop-side only |
| [`mizorewww/laya-coreml`](https://github.com/mizorewww/laya-coreml) | ANE | ~5 ms short decisions (their claim) | Apple | Same |
| TypeSafe Jev 1.13.0 cloud | HTTPS | 236–276 ms P50 published; ~314 ms with network | $0.042 / 1M input, output free | **Do not require.** LAN-first |

EdgeJev AG News (n=400, batch=1): INT8 91.2% vs FP32 92.8%. Emotion INT8 48.2% vs FP32 54.0% — there **is** a quality tax. Re-run L2 goldens on the INT8 graph before making it the household binary. Mixed precision (“decision path fp32”) is 19.2 ms / 366 MB if INT8 L2 drops.

A third-party Aar extension measured **warm CPU PyTorch** at 198–418 ms per question (8 threads). That is the number that makes EdgeJev interesting.

ONNX WebGPU ports exist (~50 ms / 3 q in-browser). Not the daemon path.

## STT (the slow hop)

Whisper is slower than Laya. Keep a **warm** [whisper.cpp](https://github.com/ggerganov/whisper.cpp) server.

| Checkpoint | Role |
| --- | --- |
| `tiny.en` | Command clips, tens of ms on CPU once warm |
| `small.en` | Better WER, still captions-class latency on GPU |
| multilingual Whisper | Only if the house is not English-first |

Hallucinated “thank you” / “subscribe” needs a **silence / energy gate** before Vikett. BuckyBoi already owns the listen window — do not double-listen.

Do **not** use LibriSpeech as a proxy for “switch to jellyfin.” Record 50 in-room clips (fan, kettle, TV). That is L3.

BuckyBoi’s own voice path is **speaker-id** (sherpa CampPlus / ERes2Net), not ASR. Vikett still needs Whisper (or equivalent) for the utterance text.

## What we actually send Laya

Criteria are **live labels**, rebuilt every call. Sketch:

```json
{
  "page": {
    "type": "choice",
    "instructions": "Which live vikett did they mean? Pick none if unsure or if they asked for a door that is not live.",
    "criteria": {
      "audio.bump/up/little": "raise default sink a little",
      "launch.app/grokbot": "open or focus Grok Bot",
      "mail.from/dave": "ask whether mail from Dave exists",
      "none": "refuse or not enough confidence"
    }
  },
  "compound": {
    "type": "noul",
    "instructions": "Does the utterance contain a second action after this take?"
  },
  "done": {
    "type": "noul",
    "instructions": "Is the requested world state already true?"
  }
}
```

Rules:

1. Never include a page that failed prune.
2. Never include walk/argv in criteria.
3. Slot values (Dave, jellyfin, little/lot) are **pre-filled by the engine** or listed as separate live ids (`mail.from/dave`). The model does not invent a contact.
4. `none` is always a legal choice. Below threshold ⇒ silence, even if `none` was not the argmax.
5. Keep `|criteria|` well under 20. If prune returns 40 live pages, **pre-rank** with the lexical gold and only send the top K plus `none`. Lexical is the shortlist; Laya is the paraphrase breaker.

## Fine-tune plan

Target: a `laya-typed-decisions`-family head specialised to Vikett.

1. Freeze L1 goldens as the oracle (this repo).
2. Expand each golden to ~20 paraphrases (human + a generator, then human-filter). Never train on the L2 holdout.
3. Negatives: every refuse string, plus live-but-wrong pages on the same snap.
4. Guest snaps: force `none` on private utterances.
5. Train `choice` over live ids; `noul` for compound and done.
6. Export ONNX → EdgeJev INT8 → L2 gate.

The split is files, not a tag on `Golden`. `ontology/train.json` holds the public paraphrases (the old `phrases.json` strings). `ontology/holdout.json` is a `Holdout` list. `Catalog` stores it beside `goldens` and does not append it. `from_golden` still tags `golden` only. `all_cases` adds tag `holdout` without calling `from_golden`. The untagged suite is the must-pass run and skips those rows. `suite --tag holdout` sees them. A tag that matches nothing still fails.

`vikett train-set --train <path> --holdout <path>` writes both files. The train file is paraphrases plus goldens plus exact suite cases. It does not contain holdout ids. `scripts/build_laya_set.py` reads that train file (and goldens, and exact cases), runs `vikett criteria`, and emits JSONL `{state, questions, answer}`. It does not open the holdout file, and it does not train. Snap id `live` is refused. Holdout eval stays `vikett suite --referee laya --tag holdout`.

Do not fine-tune so hard that lexical aliases regress — run L1 after every export.

## Recommended Omarchy box

1. Warm whisper.cpp, `small.en` or `tiny.en`, silence gate from BuckyBoi.
2. EdgeJev INT8 (or official multilingual if a GPU is free) as a `systemd --user` daemon.
3. `vikettd` unions snaps, prunes, shortlists, calls `system_one`, walks, reconciles on Hyprland socket2.
4. Fine-tune before calling it done. Zero-shot will lose to `src/engine.ts`.
5. Cloud Jev is a laptop fallback when the house is packed, never a dependency.

## Calibration / thresholds (starting points, to be measured)

| Signal | Start |
| --- | --- |
| choice confidence to walk an `act` | ≥ 0.62 and margin ≥ ~0.08 vs second |
| private `ask` | ≥ 0.70 |
| `noul` compound | ≥ 0.55 to split |
| `done` | ≥ 0.8 ⇒ skip walk (already true) |
| lexical gold confidence | same 0.62 floor (this tree) |

Wrong `act` is worse than silence. Tune false-positive walk < 1% on the adversarial refuse set before loosening.
