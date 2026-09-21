# Contributing

This is a protocol repo. Prefer a new **page** over a new cleverness.

1. Add the page to `src/catalog.ts` (id `module.verb`, aliases, enum slots, `when`, policy, examples, refuses).
2. Extend the snap only if prune needs new state.
3. Add goldens: one take, one nearby refuse. Guest-test private pages.
4. Dump JSON (`ontology/*.json` must match `src/`).
5. Do not put `hyprctl` strings in referee criteria.
6. Do not add send-mail, buy, click-pixel, or free-number slots.

```bash
node --experimental-strip-types --test tests/goldens.test.ts
```
