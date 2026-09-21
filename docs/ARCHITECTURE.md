# Architecture

## Pipeline

```
0  Presence     BuckyBoi          who / gate / listen window. Fail closed.
1  Input        whisper.cpp       utterance text. Pose names, not landmarks.
2  Snapshot     module snap()     hyprctl -j, wpctl, allowlist, mail index.
3  Prune        vikett engine     catalogue × snap × policy → live viketts.
4  Referee      Laya / lexical    choice / noul / score. Silence below threshold.
5  Walk / ask   module driver     hyprctl, wpctl, Surface API, notmuch.
6  Reconcile    socket2 / events  did fullscreen happen. Retry one edge or stop.
```

The model sits at **4**. Everything else is deterministic code.

## Process sketch (Omarchy / Hyprland)

```
buckyboi  ──sock──►  vikettd  ──hyprctl──►  Hyprland
                │         │
                │         ├── wpctl / playerctl / grim / brightnessctl
                │         ├── unix notmuch / khal (private)
                │         └── GlassSpear Surface API (scenes)
                │
         whisper-server (warm)
         edgejev / laya  (warm, systemd --user)
```

`vikettd` is not in this repo yet. This tree is the catalogue, the prune, the gold referee, and the contracts the daemon must obey.

## Module contract

A module is three functions and a list of pages:

```
snap(ctx) -> fields for this module
isLive(page, snap, slots) -> ok + why
walk(page, slots, snap) -> argv the driver already knows
ask(page, slots, snap) -> JSON matching askShape
```

The referee **must not** see `walk` strings. Labels and slot enums only.

## Prune

A page is live only if:

1. Policy allows it for `{who, occupants, guest}`.
2. `when` is true on this snap (client exists, timer running, climate entity present, …).
3. Required slots either fill from the utterance or have a legal default (`amount=little`, `target=active`).
4. The page is not reserved (`scene.handoff` is authored so tests can assert **denied** until GlassSpear ships it).

Guests: hide `policy: private` **and** the whole `mail` module, even if a mail page was marked `confirm`. Calendar is private. Notifications are private.

If the utterance **names** a pruned module (`calendar`, `email`) do **not** wander onto a live neighbour. Silence.

## Referee input

State is a compact JSON blob, rebuilt every utterance:

```json
{
  "who": "jim",
  "where": "desk",
  "guest": false,
  "utterance": "increase the volume a bit",
  "live": [
    { "id": "audio.bump", "label": "raise default sink a little" },
    { "id": "launch.app/grokbot", "label": "open or focus Grok Bot" }
  ]
}
```

Questions (see [MODELS.md](MODELS.md)):

- `page`: `choice` over live ids + `none`
- `compound`: `noul` — is there a second action
- `done`: `noul` — is the world already in the requested state

Keep the choice set **short**. Rebuild it from the prune. Do not ship the whole 50-page catalogue as criteria — that is how choice heads collapse.

## Drivers (workstation)

| Module | Snap | Walk |
| --- | --- | --- |
| wm | `hyprctl -j clients,workspaces,activewindow,monitors` | `dispatch focuswindow address:…` etc. |
| audio | `wpctl get-volume @DEFAULT_SINK@` | `wpctl set-volume -l 1.0 … 5%+` |
| launch | allowlist file + running classes | `dispatch exec` or focus |
| media | `playerctl status` | `playerctl play-pause` |
| display | `brightnessctl g` | `brightnessctl set 5%-` |
| capture | active output | `grim -o <output>` |
| bucky | sock status | `buckyboi --hide` / listen |
| scene | GlassSpear session | Surface API `apply_scene` |
| lights / climate | HA state | allowlisted entity groups |
| mail / calendar | local index, speaker-bound | ask JSON; open-last is confirm |
| timer | daemon remaining_sec | start/add/cancel |
| weather | HA weather / LAN cache | ask JSON |

LAN-first. No WAN ports. mTLS or a per-device token if GlassSpear is on another host.

## Identity seam

BuckyBoi emits something like:

```json
{
  "who": "jim",
  "gate": "FACE",
  "utterance": "mute",
  "pose": null,
  "occupants": ["jim"]
}
```

Poses are **names**: `fist` hide, `palm` listen, `peace` play/pause, `point` / `thumb` reserved. Vikett may map a pose onto a page **only if that page lists the pose** and is live. Raw landmarks never enter the referee.

UNKNOWN chip ⇒ no private pages, no owner pages.

## Workstation vs house

On a single Omarchy box, hyprctl **is** the layout driver. On a house, GlassSpear’s surface agent is the layout driver and Vikett’s `scene.*` / `wm.*` pages target the Surface API instead of raw hyprctl. Same pages. Different walk backend. The referee does not know which.
