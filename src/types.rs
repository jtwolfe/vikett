use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PageKind {
    Act,
    Ask,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Policy {
    Household,
    Private,
    Owner,
    Confirm,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SlotValue {
    pub id: String,
    pub label: String,
    pub aliases: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Slot {
    pub id: String,
    pub label: String,
    pub required: bool,
    pub values: Vec<SlotValue>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Page {
    pub id: String,
    pub module: String,
    pub kind: PageKind,
    pub title: String,
    pub summary: String,
    pub aliases: Vec<String>,
    pub slots: Vec<Slot>,
    pub when: String,
    pub policy: Policy,
    /// Separate from `Policy::Confirm`. Either one sets `EngineResult.confirm`.
    #[serde(default, skip_serializing_if = "is_false")]
    pub confirm: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub walk: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "askShape")]
    pub ask_shape: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pose: Option<String>,
    pub examples: Vec<String>,
    pub refuse: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModuleDef {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub driver: String,
    pub snap: String,
    pub priority: i32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Client {
    pub address: String,
    pub class: String,
    pub title: String,
    pub workspace: String,
    #[serde(default)]
    pub focused: bool,
    #[serde(default)]
    pub fullscreen: bool,
    #[serde(default)]
    pub floating: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NextEvent {
    pub title: String,
    #[serde(rename = "inMin")]
    pub in_min: i32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UnreadRow {
    pub count: u32,
    #[serde(default)]
    pub subjects: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Snap {
    pub id: String,
    pub title: String,
    pub who: String,
    #[serde(rename = "where")]
    pub place: String,
    pub occupants: Vec<String>,
    pub guest: bool,
    #[serde(default)]
    pub clients: Vec<Client>,
    #[serde(default)]
    pub workspaces: Vec<String>,
    #[serde(default)]
    pub active_workspace: String,
    #[serde(default)]
    pub volume: f32,
    #[serde(default)]
    pub muted: bool,
    #[serde(default)]
    pub default_sink: String,
    #[serde(default)]
    pub allowlist: Vec<String>,
    #[serde(default)]
    pub running: Vec<String>,
    #[serde(default)]
    pub mail_online: bool,
    #[serde(default)]
    pub contacts: Vec<String>,
    #[serde(default)]
    pub unread_from: BTreeMap<String, UnreadRow>,
    #[serde(default)]
    pub media_playing: bool,
    #[serde(default)]
    pub now_playing: Option<String>,
    #[serde(default)]
    pub scene: Option<String>,
    #[serde(default)]
    pub lights: f32,
    #[serde(default)]
    pub timer_sec: Option<u32>,
    #[serde(default)]
    pub bucky_listening: bool,
    #[serde(default)]
    pub brightness: f32,
    #[serde(default)]
    pub climate: Option<f32>,
    #[serde(default)]
    pub next_event: Option<NextEvent>,
    #[serde(default)]
    pub weather: Option<String>,
    /// House owner chip. Owner-policy pages require who == owner.
    #[serde(default = "default_owner")]
    pub owner: String,
    /// Connected outputs. 0 means "unspecified" and is treated as 1.
    #[serde(default)]
    pub monitors: u32,
    #[serde(default)]
    pub lock_available: bool,
    #[serde(default)]
    pub notification: Option<String>,
    #[serde(default)]
    pub network: Option<String>,
    #[serde(default)]
    pub bluetooth_on: bool,
    #[serde(default)]
    pub mic_muted: bool,
    #[serde(default)]
    pub sinks: Vec<String>,
    #[serde(default)]
    pub focused_output: Option<String>,
    /// Binaries `browser.focus` may `exec_cmd` when the window is unmapped.
    #[serde(default)]
    pub bins: Vec<String>,
    /// Closed name lists. `bookmark_folder` is discovered on the live snap.
    #[serde(default)]
    pub lists: BTreeMap<String, Vec<String>>,
    /// `browser.downloads` bucket. `None` keeps that ask dead (not `{ok:true}`).
    #[serde(default)]
    pub downloads: Option<u32>,
}

fn default_owner() -> String {
    "jim".into()
}

fn is_false(v: &bool) -> bool {
    !*v
}

impl Snap {
    pub fn owner_id(&self) -> &str {
        if self.owner.is_empty() {
            "jim"
        } else {
            &self.owner
        }
    }

    pub fn monitor_count(&self) -> u32 {
        if self.monitors == 0 {
            1
        } else {
            self.monitors
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveVikett {
    pub page_id: String,
    pub label: String,
    pub slots: BTreeMap<String, String>,
    pub why: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeadVikett {
    pub page_id: String,
    pub why: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Take {
    pub page_id: Option<String>,
    pub slots: BTreeMap<String, String>,
    pub confidence: f32,
    pub reason: String,
}

impl Take {
    pub fn silence(reason: impl Into<String>) -> Self {
        Self {
            page_id: None,
            slots: BTreeMap::new(),
            confidence: 0.0,
            reason: reason.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WalkPlan {
    pub command: String,
    pub driver: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RefereeKind {
    Lexical,
    /// Laya / EdgeJev System One (`choice` / `noul`). Not a chat LLM.
    Laya,
}

impl RefereeKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Lexical => "lexical",
            Self::Laya => "laya",
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::Lexical => Self::Laya,
            Self::Laya => Self::Lexical,
        }
    }
}

impl std::fmt::Display for RefereeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineResult {
    pub utterance: String,
    pub snap_id: String,
    pub referee: RefereeKind,
    pub live: Vec<LiveVikett>,
    pub dead: Vec<DeadVikett>,
    pub take: Take,
    pub walk: Option<WalkPlan>,
    pub answer: Option<serde_json::Value>,
    pub confirm: bool,
    pub remaining: Vec<String>,
    pub trace: crate::trace::Trace,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Golden {
    pub id: String,
    pub snap: String,
    pub utterance: String,
    pub expect_page: Option<String>,
    #[serde(default)]
    pub expect_slots: Option<BTreeMap<String, String>>,
    #[serde(default)]
    pub notes: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Phrase {
    pub id: String,
    pub golden: String,
    pub utterance: String,
    pub expect_page: Option<String>,
    #[serde(default)]
    pub notes: String,
}
