import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { GOLDENS, SNAPS } from "../src/fixtures.ts";
import { runGolden, splitCompound } from "../src/engine.ts";

describe("vikett goldens", () => {
  for (const g of GOLDENS) {
    it(g.id, () => {
      const r = runGolden(g, SNAPS);
      assert.equal(r.got, g.expectPage, `${g.id}: got ${r.got} expected ${g.expectPage} (${g.notes})`);
    });
  }

  it("splits compound", () => {
    assert.deepEqual(splitCompound("switch to jellyfin and fullscreen"), [
      "switch to jellyfin",
      "fullscreen",
    ]);
  });
});
