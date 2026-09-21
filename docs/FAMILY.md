# Family

Three jobs. One seam. Vikett is **parallel** to GlassSpear, not inside it, and not inside BuckyBoi.

```
BuckyBoi notices you
      │
   vikett offers the live doors
      │
GlassSpear / hyprctl walks through
```

| Repo | Visibility | Job |
| --- | --- | --- |
| [jtwolfe/buckyboi](https://github.com/jtwolfe/buckyboi) | public | Overlay buddy. Gaze, face, voice, gestures, gate. Emits `{who, utterance, pose}` |
| [jtwolfe/vikett](https://github.com/jtwolfe/vikett) | public | This protocol. Catalogue, prune, typed take |
| [jtwolfe/GlassSpear](https://github.com/jtwolfe/GlassSpear) | private | Scene OS. Surfaces, scenes, policy, Surface API |
| [jtwolfe/glass](https://github.com/jtwolfe/glass) | — | Android Grok Bot pipe. **Not** this |

## BuckyBoi → Vikett

BuckyBoi is a Linux-desktop overlay: rotating icosahedron, gaze-repel, dwell-to-listen if the gate allows, named gestures (`fist` / `palm` / `thumb` / `point` / `peace`). Identity is fail-closed and offline.

Vikett **does not** enroll faces, open the mic, or draw the overlay. It trusts the auth chip the way Hyprland trusts a keybind: if the chip is UNKNOWN, private pages do not exist.

Listen burst in:

```json
{ "who": "jim", "gate": "FACE", "utterance": "mute", "pose": null, "occupants": ["jim"] }
```

Pose-only burst (no utterance) may take a page that lists that pose **and is live**. `peace` → `media.play_pause`. `fist` → `bucky.hide`. `palm` is usually consumed by BuckyBoi itself as “start listen.”

BuckyBoi sock (`$XDG_RUNTIME_DIR/buckyboi.sock`) is also a **driver** for `bucky.*` pages (`--hide`, `--wake`, mute-asr).

## Vikett → GlassSpear / hyprctl

GlassSpear (concept): LAN-first scene OS. Controller, surface agents, Hyprland as layout driver, HA for presence. Intent tools are a **closed set**. Vikett is the missing typed referee in front of that closed set.

- On a workstation, `wm.*` / `launch.*` walk `hyprctl`.
- On a house, the same page ids walk GlassSpear `apply_scene` / `open_page` / `focus_pane`.
- `scene.apply` slots are **authored scene ids** (`kitchen-cook`, `living-watch`, …), never “make it cozy.”
- `scene.handoff` is reserved until GlassSpear’s handoff epic exists. Tests assert deny.

Vikett does not own the scene library. It offers doors whose slot enums **are** that library.

## Naming

- **BuckyBoi** — buckyball buddy. The body in the corner.
- **GlassSpear** — pierce chrome to apps. The house OS.
- **Vikett** — wiki + wicket. The small door.

Keep the jokes. They are load-bearing: they stop the layers collapsing into “the AI.”
