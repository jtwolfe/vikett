export type PageKind = "act" | "ask";
export type Policy = "household" | "private" | "owner" | "confirm";

export type SlotValue = {
  id: string;
  label: string;
  aliases: string[];
};

export type Slot = {
  id: string;
  label: string;
  required: boolean;
  values: SlotValue[];
};

export type Page = {
  id: string;
  module: string;
  kind: PageKind;
  title: string;
  summary: string;
  aliases: string[];
  slots: Slot[];
  when: string;
  policy: Policy;
  walk?: string;
  askShape?: string;
  pose?: string;
  examples: string[];
  refuse: string[];
};

export type ModuleDef = {
  id: string;
  title: string;
  summary: string;
  driver: string;
  snap: string;
  priority: number;
};

export type Client = {
  address: string;
  class: string;
  title: string;
  workspace: string;
  focused: boolean;
  fullscreen: boolean;
  floating: boolean;
};

export type NextEvent = {
  title: string;
  inMin: number;
};

export type Snap = {
  id: string;
  title: string;
  who: string;
  where: string;
  occupants: string[];
  guest: boolean;
  clients: Client[];
  workspaces: string[];
  activeWorkspace: string;
  volume: number;
  muted: boolean;
  defaultSink: string;
  allowlist: string[];
  running: string[];
  mailOnline: boolean;
  contacts: string[];
  unreadFrom: Record<string, { count: number; subjects: string[] }>;
  mediaPlaying: boolean;
  nowPlaying: string | null;
  scene: string | null;
  lights: number;
  timerSec: number | null;
  buckyListening: boolean;
  brightness: number;
  climate: number | null;
  nextEvent: NextEvent | null;
  weather: string | null;
};

export type LiveVikett = {
  pageId: string;
  label: string;
  slots: Record<string, string>;
  why: string;
};

export type Take = {
  pageId: string | null;
  slots: Record<string, string>;
  confidence: number;
  reason: string;
};

export type EngineResult = {
  utterance: string;
  live: LiveVikett[];
  take: Take;
  walk: string | null;
  answer: unknown | null;
};
