import { PAGES } from "./catalog.ts";
import type { EngineResult, LiveVikett, Page, Snap, Take } from "./types.ts";

const STOP = new Set([
  "a",
  "the",
  "to",
  "on",
  "in",
  "it",
  "me",
  "i",
  "am",
  "is",
  "are",
  "any",
  "of",
  "and",
  "please",
  "up",
  "my",
]);

function norm(s: string) {
  return s
    .toLowerCase()
    .replace(/[’']/g, "'")
    .replace(/[^a-z0-9+ ]+/g, " ")
    .replace(/\s+/g, " ")
    .trim();
}

function tokens(s: string) {
  return norm(s)
    .split(" ")
    .filter((t) => t && !STOP.has(t));
}

function containsPhrase(hay: string, needle: string) {
  const h = ` ${norm(hay)} `;
  const n = ` ${norm(needle)} `;
  return h.includes(n);
}

export function splitCompound(utterance: string): string[] {
  const n = norm(utterance);
  const parts = n.split(/\band then\b|\band\b/).map((p) => p.trim()).filter(Boolean);
  return parts.length > 1 ? parts : [utterance];
}

function slotFill(page: Page, utterance: string): { slots: Record<string, string>; missing: string[] } {
  const slots: Record<string, string> = {};
  const missing: string[] = [];
  for (const slot of page.slots) {
    let hit: string | null = null;
    let best = 0;
    for (const v of slot.values) {
      for (const a of [v.id, v.label, ...v.aliases]) {
        if (!containsPhrase(utterance, a) && !tokens(utterance).includes(norm(a))) continue;
        const score = a.length;
        if (score >= best) {
          best = score;
          hit = v.id;
        }
      }
    }
    if (hit) slots[slot.id] = hit;
    else if (slot.required) missing.push(slot.id);
    else if (slot.id === "target" && containsPhrase(utterance, "this")) slots.target = "active";
    else if (slot.id === "amount") slots.amount = "little";
  }
  return { slots, missing };
}

function clientMatch(snap: Snap, target: string | undefined) {
  if (!target || target === "active") return snap.clients.find((c) => c.focused) ?? snap.clients[0];
  const t = target.toLowerCase();
  return snap.clients.find(
    (c) => c.class.toLowerCase().includes(t) || c.title.toLowerCase().includes(t),
  );
}

export function isLive(page: Page, snap: Snap, slots: Record<string, string>): { ok: boolean; why: string } {
  if ((page.policy === "private" || page.module === "mail") && snap.guest) {
    return { ok: false, why: "private pages hidden — guest present" };
  }

  switch (page.id) {
    case "wm.focus": {
      const c = clientMatch(snap, slots.target);
      if (c) return { ok: true, why: `client ${c.class}` };
      return { ok: false, why: "no matching client" };
    }
    case "wm.fullscreen": {
      const c = clientMatch(snap, slots.target ?? "active");
      if (!c) return { ok: false, why: "no client" };
      if (c.fullscreen) return { ok: false, why: "already fullscreen" };
      return { ok: true, why: `can fullscreen ${c.class}` };
    }
    case "wm.move_ws":
    case "wm.float":
    case "wm.close":
      return snap.clients.some((c) => c.focused)
        ? { ok: true, why: "focused client" }
        : { ok: false, why: "nothing focused" };
    case "wm.workspace":
      return { ok: true, why: "workspace switch is always legal" };
    case "audio.bump": {
      if (slots.direction === "up" && snap.volume >= 0.99) return { ok: false, why: "already max" };
      return { ok: true, why: `sink ${snap.defaultSink} @ ${snap.volume}` };
    }
    case "audio.mute":
      return snap.muted ? { ok: false, why: "already muted" } : { ok: true, why: "can mute" };
    case "audio.unmute":
      return snap.muted ? { ok: true, why: "muted" } : { ok: false, why: "not muted" };
    case "audio.headphones":
      return { ok: true, why: "sink switch offered" };
    case "audio.speakers":
      return { ok: true, why: "sink switch offered" };
    case "launch.app": {
      const app = slots.app;
      if (!app) return { ok: false, why: "no app slot" };
      if (!snap.allowlist.includes(app)) return { ok: false, why: "not allowlisted" };
      return { ok: true, why: snap.running.includes(app) ? "already running → focus" : "exec" };
    }
    case "mail.from":
    case "mail.unread":
    case "mail.flag":
    case "mail.open_last":
      if (!snap.mailOnline) return { ok: false, why: "mail offline" };
      return { ok: true, why: "account bound" };
    case "media.play_pause":
    case "media.next":
    case "media.prev":
    case "media.ask_now":
      return snap.nowPlaying || snap.mediaPlaying
        ? { ok: true, why: snap.nowPlaying ?? "player" }
        : { ok: false, why: "no player" };
    case "timer.add":
    case "timer.ask":
    case "timer.cancel":
      return snap.timerSec != null ? { ok: true, why: `${snap.timerSec}s left` } : { ok: false, why: "no timer" };
    case "timer.start":
      return { ok: true, why: "timer daemon" };
    case "bucky.hide":
      return { ok: true, why: "overlay" };
    case "scene.handoff":
      return { ok: false, why: "handoff driver not wired (page reserved)" };
    case "wm.split_beside":
      return { ok: true, why: "allowlisted split" };
    case "calendar.ask_next":
    case "calendar.ask_today":
      return { ok: true, why: snap.nextEvent ? snap.nextEvent.title : "calendar empty" };
    case "display.bump":
    case "display.night":
      return { ok: true, why: `panel ${snap.brightness}` };
    case "climate.bump":
    case "climate.off":
      return snap.climate != null ? { ok: true, why: `${snap.climate}C` } : { ok: false, why: "no climate in this room" };
    case "capture.screenshot":
      return { ok: true, why: "focused output" };
    case "weather.ask":
      return snap.weather ? { ok: true, why: snap.weather } : { ok: false, why: "weather offline" };
    default:
      return { ok: true, why: "default live" };
  }
}

function aliasScore(page: Page, utterance: string): number {
  let best = 0;
  for (const a of page.aliases) {
    if (containsPhrase(utterance, a)) best = Math.max(best, 8 + a.length);
  }
  const uTok = new Set(tokens(utterance));
  for (const a of page.aliases) {
    const aTok = tokens(a);
    const hits = aTok.filter((t) => uTok.has(t));
    if (hits.length >= 2 || (hits.length === 1 && aTok.length === 1 && hits[0].length >= 4)) {
      best = Math.max(best, 3 * hits.length);
    }
  }
  return best;
}

export function liveViketts(snap: Snap, utterance?: string): LiveVikett[] {
  const out: LiveVikett[] = [];
  for (const page of PAGES) {
    const { slots } = utterance ? slotFill(page, utterance) : { slots: {} as Record<string, string> };
    const live = isLive(page, snap, slots);
    if (!live.ok) continue;
    out.push({
      pageId: page.id,
      label: page.title,
      slots,
      why: live.why,
    });
  }
  return out;
}

export function decide(utterance: string, snap: Snap): EngineResult {
  const n = norm(utterance);
  if (
    /\b(email|mail|message)\b.*\b(that|to say|saying)\b|\bsend\b|\bbuy\b|\bclick\b|\bpurchase\b|\bschedule\b|\binvite\b|\b\d+(\.\d+)?\s*(percent|%|degrees|celsius|kelvin)\b/.test(
      n,
    )
  ) {
    const live = liveViketts(snap, utterance);
    return {
      utterance,
      live,
      take: { pageId: null, slots: {}, confidence: 0, reason: "refused — no page for compose/send/buy/click" },
      walk: null,
      answer: null,
    };
  }

  const live = liveViketts(snap, utterance);
  const liveIds = new Set(live.map((l) => l.pageId));
  const liveMods = new Set(PAGES.filter((p) => liveIds.has(p.id)).map((p) => p.module));

  const moduleHints: { re: RegExp; module: string }[] = [
    { re: /\b(calendar|appointment|meetings?)\b/, module: "calendar" },
    { re: /\b(email|emails|inbox)\b/, module: "mail" },
  ];
  for (const h of moduleHints) {
    if (h.re.test(n) && !liveMods.has(h.module)) {
      return {
        utterance,
        live,
        take: {
          pageId: null,
          slots: {},
          confidence: 0,
          reason: `${h.module} not live — silence rather than a different door`,
        },
        walk: null,
        answer: null,
      };
    }
  }

  const scored: { page: Page; score: number; slots: Record<string, string> }[] = [];
  for (const page of PAGES) {
    if (!liveIds.has(page.id)) continue;
    const { slots, missing } = slotFill(page, utterance);
    if (missing.length) continue;
    let a = aliasScore(page, utterance);
    if (page.id === "launch.app" && /\b(switch to|go to|focus|show me|bring up)\b/i.test(utterance) && slots.app) {
      a = Math.max(a, 8);
    }
    if (a <= 0) continue;
    let score = a;
    for (const [k, v] of Object.entries(slots)) {
      if (v) score += 4 + k.length;
    }
    if (page.kind === "ask" && /\b(have i|any|what|how|is|where|which|who)\b/i.test(utterance)) {
      score += 6;
    }
    if (page.id === "launch.app" && /\b(switch to|go to|focus|show me|bring up)\b/i.test(utterance)) {
      const app = slots.app;
      if (app && !snap.running.includes(app) && snap.allowlist.includes(app)) score += 20;
    }
    scored.push({ page, score, slots });
  }

  scored.sort((x, y) => y.score - x.score);

  const none: Take = { pageId: null, slots: {}, confidence: 0, reason: "no live door crossed threshold" };

  if (!scored.length) {
    return { utterance, live, take: none, walk: null, answer: null };
  }

  const top = scored[0];
  const second = scored[1];
  if (second && top.score - second.score < 3 && top.page.module !== second.page.module) {
    return {
      utterance,
      live,
      take: {
        pageId: null,
        slots: {},
        confidence: 0.4,
        reason: `ambiguous ${top.page.id} vs ${second.page.id}`,
      },
      walk: null,
      answer: null,
    };
  }

  const lv = live.find((l) => l.pageId === top.page.id);
  const take: Take = {
    pageId: top.page.id,
    slots: { ...lv?.slots, ...top.slots },
    confidence: Math.min(0.98, 0.55 + top.score / 40),
    reason: "lexical referee (gold baseline — not Laya)",
  };

  if (take.confidence < 0.62) {
    return {
      utterance,
      live,
      take: { pageId: null, slots: {}, confidence: take.confidence, reason: "below threshold" },
      walk: null,
      answer: null,
    };
  }

  const page = PAGES.find((p) => p.id === take.pageId)!;
  let walk: string | null = null;
  let answer: unknown = null;
  if (page.kind === "act") walk = fillWalk(page, take.slots, snap);
  else answer = fillAsk(page, take.slots, snap);

  return { utterance, live, take, walk, answer };
}

function fillWalk(page: Page, slots: Record<string, string>, snap: Snap): string {
  switch (page.id) {
    case "audio.bump": {
      const dir = slots.direction === "down" ? "-" : "+";
      const step = slots.amount === "lot" ? "10%" : "5%";
      return `wpctl set-volume -l 1.0 @DEFAULT_SINK@ ${step}${dir}`;
    }
    case "audio.mute":
      return "wpctl set-mute @DEFAULT_SINK@ 1";
    case "audio.unmute":
      return "wpctl set-mute @DEFAULT_SINK@ 0";
    case "wm.focus": {
      const c = clientMatch(snap, slots.target);
      return c ? `hyprctl dispatch focuswindow address:${c.address}` : page.walk ?? "";
    }
    case "wm.fullscreen":
      return "hyprctl dispatch fullscreen 1";
    case "wm.workspace":
      return `hyprctl dispatch workspace ${slots.ws}`;
    case "wm.move_ws":
      return `hyprctl dispatch movetoworkspace ${slots.ws}`;
    case "launch.app": {
      const app = slots.app;
      if (app && snap.running.includes(app)) {
        const c = clientMatch(snap, app);
        return c
          ? `hyprctl dispatch focuswindow address:${c.address}`
          : `focus ${app}`;
      }
      return `hyprctl dispatch exec ${app}`;
    }
    case "display.bump": {
      const dir = slots.direction === "down" ? "-" : "+";
      const step = slots.amount === "lot" ? "10%" : "5%";
      return `brightnessctl set ${step}${dir}`;
    }
    case "display.night":
      return "hyprctl hyprsunset temperature 3500";
    case "climate.bump": {
      const step = slots.direction === "down" ? -1 : 1;
      return `climate.set_temperature step ${step}`;
    }
    case "climate.off":
      return "climate.turn_off";
    case "capture.screenshot":
      return "grim -o focused ~/Pictures/vikett.png";
    default:
      return page.walk ?? page.id;
  }
}

function fillAsk(page: Page, slots: Record<string, string>, snap: Snap): unknown {
  switch (page.id) {
    case "mail.from": {
      const who = slots.who ?? "anyone";
      const row = snap.unreadFrom[who] ?? { count: 0, subjects: [] };
      return { count: row.count, subjects: row.subjects, who };
    }
    case "mail.unread": {
      const count = Object.values(snap.unreadFrom).reduce((n, r) => n + r.count, 0);
      return { unread: count };
    }
    case "audio.ask_level":
      return { volume: snap.volume, muted: snap.muted, sink: snap.defaultSink };
    case "wm.ask_focused": {
      const c = snap.clients.find((x) => x.focused);
      return c ?? { none: true };
    }
    case "launch.ask_running":
      return { running: snap.running.includes(slots.app), app: slots.app };
    case "timer.ask":
      return { remaining_sec: snap.timerSec };
    case "media.ask_now":
      return { title: snap.nowPlaying, playing: snap.mediaPlaying };
    case "scene.ask_active":
      return { scene: snap.scene, where: snap.where, occupants: snap.occupants };
    case "bucky.ask_who":
      return { who: snap.who, occupants: snap.occupants, guest: snap.guest };
    case "calendar.ask_next":
      return snap.nextEvent ?? { none: true };
    case "calendar.ask_today":
      return { remaining: snap.nextEvent ? 1 : 0 };
    case "weather.ask":
      return { condition: snap.weather };
    default:
      return { ok: true };
  }
}

export function runGolden(g: { utterance: string; snap: string; expectPage: string | null }, snaps: Snap[]) {
  const snap = snaps.find((s) => s.id === g.snap);
  if (!snap) return { pass: false, got: "missing-snap" };
  const parts = splitCompound(g.utterance);
  const first = decide(parts[0], snap);
  const got = first.take.pageId;
  return { pass: got === g.expectPage, got, take: first.take };
}
