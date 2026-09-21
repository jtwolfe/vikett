# Concept

Vikett is a **control protocol**, not an app and not a model.

The joke is the name: **wiki** (a catalogue of pages) + **wicket** (a small gate you actually walk through). German W→V, and a nod to Old Norse *vík*. It is allowed to stay a private joke.

## The failure mode it is for

A warm Whisper hop into an LLM that prints:

```
hyprctl dispatch focuswindow class:jellyfin
hyprctl dispatch fullscreen 1
```

works in a demo and then:

1. Invents a class that is not mapped.
2. Fullscreens the wrong client.
3. Reads mail out loud because a guest said “Dave.”
4. Sets volume to 37% because the model likes numbers.
5. Clicks a pixel it hallucinated from a screenshot.

The compositor, the inbox, and the thermostat are **closed machines**. Freeform tool-use is the wrong interface.

## One sentence

**Humans author doors. The world prunes them. A typed referee picks one. A driver walks. Silence is allowed.**

## Words

| Word | Meaning |
| --- | --- |
| **page** | One authored door (`wm.focus`, `audio.bump`, `mail.from`) |
| **module** | Pages + `snap()` + walk/ask driver. No shell from the model |
| **snap** | Structured state (hyprctl JSON, wpctl, allowlist, occupants) |
| **live viketts** | Catalogue after snap × policy. Dead apps grow no limbs |
| **take** | Referee output: `pageId` + enum slots + confidence, or silence |
| **walk** | Driver executes an `act` |
| **ask** | Structured facts, not a chat summary |
| **pose** | Named BuckyBoi gesture (`fist`, `palm`, `peace`) — not raw landmarks |

Full glossary: [GLOSSARY.md](GLOSSARY.md).

## Everyday activities (the ontology hypothesis)

What people actually say at a desk, in a kitchen, in a living room with a guest:

- Switch, fullscreen, other monitor, put this on three.
- A bit louder. Headphones. Mute.
- Open Grok Bot. Is it already open?
- Mail from Dave? Don’t read it. Don’t send.
- Pause the show. What’s playing. Put it on the TV (page reserved until GlassSpear handoff exists).
- Kitchen cook. Someone’s coming — shared only.
- How long on the oven. Add five minutes.
- What’s next on my calendar (not while a guest is in frame).
- Dim the screen. Night light. A bit warmer. Screenshot.
- Is it going to rain (condition, not packing advice).

Each of those is a page or a silence. “Email Dave that I’ll be late,” “buy the thing in that tab,” “make it look nicer,” “set it to 37%” are **refuses**.

## Policy

| Policy | Who | Example |
| --- | --- | --- |
| `household` | Anyone the gate lets speak | volume, scenes, focus |
| `private` | Speaker, no guest occupants | mail, calendar, last notification |
| `owner` | House owner chip | guest mode, lock private |
| `confirm` | Live, but walk waits on a yes | close window, open Dave’s last, screenshot with guest |

Fail closed. A missing snap field does not default to “probably fine.”

## Compound speech

“Switch to jellyfin and fullscreen” is **two takes**. Split on `and` / `and then`. Each take is pruned and refereed independently. The second take sees the world **after** the first walk, once reconcile has run. The goldens only assert the first take — the second is an integration test for the daemon.

## What Vikett is not

It is not GlassSpear. GlassSpear owns surfaces, scenes, and the house. Vikett **offers** `scene.apply` as a door; GlassSpear **walks** `apply_scene`.

It is not BuckyBoi. BuckyBoi owns gaze, face, voice, gestures, and the gate. Vikett consumes `{who, utterance, pose}`.

It is not [jtwolfe/glass](https://github.com/jtwolfe/glass). That is an Android Grok Bot pipe.

It is not an LLM with tools. Laya answers typed questions over live labels. Drivers own argv.
