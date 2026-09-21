# Glossary

| Term | Meaning |
| --- | --- |
| **page** | One authored door. Id like `wm.focus`. Kind `act` or `ask` |
| **module** | A group of pages plus snap + driver |
| **catalogue** | The full authored set. Humans write it |
| **snap** | Structured world state at decide-time |
| **live viketts** | Catalogue after snap × policy |
| **prune** | The function that produces live viketts |
| **take** | Referee pick: page + enum slots + confidence, or silence |
| **walk** | Driver execution of an `act` |
| **ask** | Structured JSON for an `ask` page |
| **slot** | Named enum. Never a free string or a model-invented number |
| **notch** | `little` / `lot` (5% / 10%, or ±1 °C). Not 37 |
| **policy** | `household` \| `private` \| `owner` \| `confirm` |
| **guest** | Occupant who is not the speaker; hides private + mail |
| **pose** | Named BuckyBoi gesture, not landmarks |
| **chip** | BuckyBoi auth display: a name or UNKNOWN |
| **gate** | BuckyBoi who-may-listen: OFF/FACE/VOICE/ANY/ALL |
| **referee** | Lexical gold or Laya. Chooses among live labels |
| **silence** | Take with `pageId: null`. First-class, not an error |
| **compound** | Utterance with two actions; sequential takes |
| **reconcile** | Check the world after a walk; retry one edge or stop |
| **allowlist** | Apps that `launch.app` may exec |
| **driver** | Code that already knows argv. Model never writes it |
| **GlassSpear** | House scene OS. Walk backend for scenes |
| **BuckyBoi** | Overlay buddy. Presence + identity |
| **Laya** | Open System 1 typed-decision model (choice/score/noul) |
| **noul** | Laya boolean: calibrated P(true) |
| **gold / golden** | Authored utterance → expected page on a named snap |
