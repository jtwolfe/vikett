# Contributing

This is a protocol repo. Prefer a new **page** over a new cleverness.

1. Add v0 pages to `ontology/pages.json`, or research extras to `src/ontology/extras.rs` (id `module.verb`, aliases, enum slots, `when`, policy, examples, refuses).
2. Extend the snap only if prune needs new state.
3. Add goldens: one take, one nearby refuse. Guest-test private pages. Assert `expectSlots` when notches/contacts matter.
4. Prune lives in `src/prune.rs` (family modules dispatch to `src/drivers/`). Walks live in `src/walk.rs`. The referee must not see argv.
5. Do not add send-mail, buy, click-pixel, or free-number slots.

L1 is the Rust lexical suite (`cargo test` / `vikett suite`): `must_fail = 0` and `wrong_act = 0`. Node goldens are a v0 lock of `src/fixtures.ts`, not that gate.

```bash
cargo test
cargo run --release -- suite --referee lexical
node --experimental-strip-types --test tests/goldens.test.ts
```
