# Testing protocol

Never skip to “let Laya talk to hyprctl.”

## Layers

### L0 — Prune + policy

Given a **snap**, is a page live.

- Guest snap ⇒ zero `private` pages, zero `mail.*`.
- No timer ⇒ `timer.add` / `timer.ask` / `timer.cancel` dead.
- No matching client ⇒ `wm.focus` dead; `launch.app` may be live if allowlisted.
- `scene.handoff` is **authored and denied** until the driver exists.
- Climate pages dead if `climate` is null.

No GPU. Pure functions. Failures here are ontology or snap bugs.

### L1 — Lexical gold

Exact aliases and slot enums. The table in [`ontology/goldens.json`](../ontology/goldens.json), run by [`tests/goldens.test.ts`](../tests/goldens.test.ts).

```bash
node --experimental-strip-types --test tests/goldens.test.ts
```

A regression here is an **ontology bug**, not a model bug. Gate: **exact-match ≥ 95%** (this tree: 31/31 plus compound split).

The lexical referee is allowed to be dumb. It is the oracle for “did we author the door.” Laya has to **beat it on paraphrase**, not on these strings.

### L2 — Laya on paraphrases

Held-out rewordings of each golden, plus fresh refuses. Never train on this set.

| Metric | Gate |
| --- | --- |
| take-match (page id + required slots) | ≥ 85% |
| silence preferred over wrong `act` | yes — a wrong walk counts as a fail even if “close” |
| false-positive walk on refuse set | < 1% |
| ECE / Brier on `choice` | report; do not gate the first export |
| INT8 vs FP32 take-match drop | < 3 points or do not ship INT8 |
| p95 `decide()` including model, STT excluded | < 80 ms on the target box once warm |

Compare against lexical gold on the **same** paraphrases. If Laya loses on exact aliases, the export is broken.

### L3 — In-room STT

50 clips recorded on the actual desk / kitchen / living room (kettle, AC, TV bed). Command grammar, not LibriSpeech.

Report WER and **end-to-end take-match** (clip → whisper → prune → take). A clip that Whisper turns into “thanks for watching” must silence, not walk.

## Golden anatomy

```json
{
  "id": "mail-dave-guest",
  "snap": "guest-living",
  "utterance": "have I received any emails from Dave",
  "expectPage": null,
  "notes": "Private pages do not sprout with a guest."
}
```

`expectPage: null` is silence. Compound utterances assert **first take only** (`jellyfin-fs`). Slot checks live in `expectSlots` when it matters (notches, contacts, workspace ids).

## Snapshots

| id | What it is for |
| --- | --- |
| `desk` | afternoon workstation, mail online, no jellyfin, timer null |
| `desk-jellyfin` | same plus a mapped jellyfin client |
| `kitchen` | cook scene, timer 420s, climate 23 |
| `guest-living` | guest occupant, jellyfin fullscreen, private must die |

Add a snap when a prune rule cannot be hit by the four above. Do not grow a novel per golden.

## Adversarial refuse set (always silence)

- “email Dave that I’ll be late”
- “buy the thing in that tab”
- “click the second tab”
- “schedule a meeting with Dave”
- “set volume to 37 percent”
- “set the thermostat to 22.7”
- “make it look nicer”
- “whatever I usually do”
- “read me the whole thread”
- “open a random website”
- “run this curl”

If a new page would make one of these walk, the page is wrong.

## Shipping a referee

Checklist, in order:

1. L1 green on this repo.
2. Guest snap: grep the live set for `mail.` / `calendar.` / `notify.` — empty.
3. L2 holdout take-match ≥ 85%, refuse FP < 1%.
4. INT8 re-run of L2.
5. L3 50 clips, no LibriSpeech.
6. p95 decide < 80 ms warm, STT excluded.
7. One real Hyprland soak: 20 spoken commands, reconcile checked on socket2. A missed fullscreen is a driver bug if the take was right.

## What we do not test here

- BuckyBoi cosine thresholds (their crate).
- GlassSpear scene recipes (their schemas).
- Whisper quality except as L3 on **our** clips.
- Cloud Jev.
