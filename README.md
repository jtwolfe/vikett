# Vikett

**vick-ett.** Wiki-shaped catalogue, wicket-shaped step.

Closed-door **control protocol**: authored pages, live prune, a typed take, then a walk. Parallel to [GlassSpear](https://github.com/jtwolfe/GlassSpear), plugged into [BuckyBoi](https://github.com/jtwolfe/buckyboi). Not a chatbot. Not a planner. The model never invents `hyprctl`, a shell line, or a percent.

```
BuckyBoi notices you
      │
   vikett offers the live doors
      │
GlassSpear / hyprctl walks through
```

| Layer | Job | Does not |
| --- | --- | --- |
| **BuckyBoi** | Who, gate, listen window, named pose | Walk a door |
| **Vikett** | Catalogue × snap × policy → take or silence | Emit argv |
| **GlassSpear / hyprctl** | Scene apply, window dispatch, HA adapters | Guess intent |

This repository is the **protocol**: ontology, schemas, lexical gold referee, model-wiring notes, test gates. It is not the Linux daemon and not the overlay buddy.

## Why a door, not a planner

Voice-to-Hyprland demos usually ask an LLM to invent a command. That is the wrong shape:

- A compositor is a closed machine. `hyprctl dispatch` strings are not a language model’s job.
- Pixels are not pages. “Click the second tab” has no legal walk.
- Guests change the legal set. Mail must not sprout because someone said “Dave.”
- Compound speech (“switch to jellyfin and fullscreen”) is two takes, not one generated script.

Vikett authors every legal move as a **page**. A snapshot of the world **prunes** the catalogue to **live viketts**. A referee (lexical gold, later Laya) **takes** one page and fills **slots from enums**. A **module driver** walks. Silence is a first-class result.

## Everyday ontology (v0)

**15 modules, 50 pages** in the v0 JSON, plus research extras in Rust (`session`, `network`, `bluetooth`, extra `wm`/`audio`/`capture` doors). Humans author them. The model only chooses.

| Module | Typical doors | Hard refuse |
| --- | --- | --- |
| `wm` | focus, fullscreen, workspace, close, split-ratio notch, group next, dwindle or master | click-by-pixel, “the red one”. `special workspace` stays a workspace |
| `audio` | bump ±5%, mute, headphones, speakers | “set volume to 37%” |
| `launch` | open allowlisted app, or focus if running | random URL, curl |
| `mail` | from Dave / unread (ask); open last (confirm) | send, compose, “email Dave that…” |
| `media` | play/pause, skip, now playing | pick a title from a screenshot |
| `bucky` | hide buddy, listen, mic mute, who is here | enroll a stranger |
| `scene` | kitchen-cook, living-watch, guest mode, lock private | “make it cozy” |
| `notify` | last ping (private), dismiss, dismiss all (confirm) | arbitrary D-Bus |
| `lights` | notch brightness, room off | free hue / entity id |
| `timer` | 5/10/15/20 min, add five, how long | “wake me at seven” |
| `calendar` | what’s next, how busy (private) | schedule / invite |
| `display` | dim the screen, night light, per-output brightness notch (`hdmi` / `edp`) | free kelvin / percent / resolution / refresh. Layout presets stay reserved |
| `climate` | warmer/colder one degree, air off | “set to 22.7” |
| `capture` | screenshot, region (grim+slurp) | email the PNG |
| `weather` | condition string | packing advice |
| `session` | lock (hyprlock), screens off (dpms), owner+confirm suspend / reboot / poweroff | idle inhibit (`force_idle` is not an inhibitor). Guests are not the owner |
| `network` | am I online; connect a named VPN (confirm) | turn off wifi, join an SSID, change DNS |
| `bluetooth` | is bluetooth on; connect or disconnect an allowlisted device | pair |
| `power` | power-saver / balanced / performance; battery bucket | fan percent. Battery ask is dead when the fraction is missing |
| `disk` | free-space bucket; timeshift create (owner + confirm) | format, delete snapshots |
| `updates` | pending bucket; full upgrade (owner + confirm) | upgrade a named package |

Amounts are **notches** (`little` / `lot`), never model-invented numbers. Slots are **enums**. Policy is `household` | `private` | `owner` | `confirm`. Guests hide `private` and the whole `mail` module.

JSON dumps: [`ontology/pages.json`](ontology/pages.json), [`ontology/modules.json`](ontology/modules.json), [`ontology/goldens.json`](ontology/goldens.json).

## Model wiring (research, 2026-09)

Laya is a **System 1 decision model**: state in, typed questions (`choice` / `score` / `noul`) out, **no generated tokens**. That is the referee we want. It is **not** a tool-calling LLM.

Published numbers we are wiring against (not our benches):

| Checkpoint / runtime | What it is | Latency we care about | Note |
| --- | --- | --- | --- |
| [`convaiinnovations/laya`](https://huggingface.co/convaiinnovations/laya) | ModernBERT-large 421M EN | 32.8 ms P50 / 1 q on T4 | Zero-shot typed-decisions **0.36** |
| `laya-multilingual` | mmBERT-base 322M | ~2.2× faster, 1024 ctx | Default Linux referee |
| `laya-typed-decisions` | 421M fine-tune | same family | **0.766** vs Jev 0.727 — **fine-tune on our goldens** |
| [`yzfly/edgejev`](https://github.com/yzfly/edgejev) | ONNX INT8, no torch | **15.6 ms** INT8 on 4 vCPU Xeon, 324 MB | Best Omarchy CPU path |
| [`@receptron/laya`](https://github.com/receptron/laya) | Node ONNX | ~140 ms / 3 q Apple CPU warm | If the engine stays TypeScript |
| [`mizorewww/laya-mlx`](https://github.com/mizorewww/laya-mlx) | Apple MLX | 7–14 ms M3 Max | **Not** a Hyprland box |
| TypeSafe Jev 1.13.0 | closed cloud | 236–276 ms P50, ~314 ms with net | LAN-first: do not require |
| whisper.cpp `tiny.en` / `small.en` | STT | tens–hundreds of ms | **The slow hop.** Keep a warm server |

**Do not** put Laya in the Hyprland frame loop. Speech rate is enough. Laya only sees **live page labels**, never `hyprctl`. Criteria lists are rebuilt every call from the prune. Keep options well under 20.

Zero-shot Laya will **lose** to the lexical gold on exact aliases and **must** be fine-tuned (the typed-decisions paper is explicit: base ~0.35, fine-tune 0.766). Detail: [docs/MODELS.md](docs/MODELS.md).

## Testing protocol

Three layers, in order. Never skip to “let Laya talk to hyprctl.”

| Layer | What | Gate |
| --- | --- | --- |
| **L0** | Prune + policy on a snap | Guest ⇒ zero private/mail leaks. No timer ⇒ `timer.add` dead |
| **L1** | Lexical gold on authored aliases | Exact-match ≥ 95% — ontology bug if it fails |
| **L2** | Laya on held-out paraphrases | Take-match ≥ 85%; prefer silence over a wrong act; false-positive walk < 1% |
| **L3** | In-room STT clips | WER on a 50-clip command set. Not LibriSpeech |

This tree ships L0+L1 in both the original TypeScript gold referee and the Rust interpreter.

```bash
cargo test
cargo run --release -- test
./scripts/run_prompt_suite.sh                 # 200+ prompts, lexical gate + Laya compare
cargo run --release -- suite --tag refuse
cargo run --release -- suite --referee laya --compare
```

The original TypeScript gate still runs:

```bash
node --experimental-strip-types --test tests/goldens.test.ts
```

Full protocol: [docs/TESTING.md](docs/TESTING.md). Fixtures: [`ontology/goldens.json`](ontology/goldens.json).

## Test rig (Rust)

The interpreter, decision tree, and TUI live in Rust. Natural language in; prune + referee + expected walk out. Nothing is dispatched unless you copy the walk yourself.

```bash
cargo run --release                  # TUI
cargo run --release -- tree -u "increase the volume a bit" -s desk
cargo run --release -- tree -u "focus zen" --live
cargo run --release -- eval -u "mute" -s desk
```

| Referee | When |
| --- | --- |
| `lexical` | Default. Alias + slot gold. No network. Always on. |
| `laya` | Typed `choice`/`noul` over live labels. POST `VIKETT_LAYA_URL` (default `http://127.0.0.1:8009/v1/systemone`). Not a chat LLM. |

TUI keys: type a phrase, Enter to take, `C-s` cycle fixture snap, `C-l` live Hyprland snap, `C-r` lexical/laya, `C-g` next golden, `C-t` run goldens, `C-p` paraphrase.

Bring Laya up (once), then the Rust binary only HTTP-calls it:

```bash
# Preferred Linux CPU path — ONNX INT8, no torch at runtime (~15 ms/q published)
# Torch 2.14 dynamo export needs onnxscript; EdgeJev 0.3.2 does not declare it.
./scripts/setup_edgejev.sh
edgejev serve --model ./jev-int8 --port 8009

# Or official PyTorch Laya if you already have torch (slower on CPU)
pip install laya
python scripts/laya_sidecar.py --port 8009
```

Research that shaped the extra pages (Hyprland 0.56.2 + this workstation): `hyprctl dispatch` for focus/fullscreen/workspace/pin/center/cyclenext/movefocus/dpms; `wpctl` for sink/source mute and HDMI vs analog; `grim`+`slurp` for capture; `hyprlock` / `hyprsunset`; `nmcli` and `bluetoothctl` as **ask-only**. Classes on this box are `foot` (terminal) and `zen` (browser).

## Layout

```
docs/           CONCEPT ARCHITECTURE MODELS TESTING FAMILY ONTOLOGY GLOSSARY
ontology/       pages.json modules.json goldens.json snaps.json phrases.json
schemas/        page snap take JSON Schema
src/            Rust interpreter (prune, lexical, model, TUI, host snap)
src/*.ts        original TypeScript gold referee
tests/          cargo tests + node:test goldens
```

## Non-goals

- A general assistant that “just does it.”
- Model-emitted shell, `hyprctl`, or HA entity ids.
- Cloud-required inference.
- Send mail, buy things, click pixels, schedule meetings.
- Putting identity in Vikett (that is BuckyBoi).
- Putting scene recipes in Vikett (that is GlassSpear).

## License

MIT. © 2026 Jim Wolfe.
