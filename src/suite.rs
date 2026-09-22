//! Extensive prompt suite: goldens + authored coverage + refuses + policy.
//!
//! Lexical cases without the `paraphrase` tag are a hard gate.
//! Laya misses into silence are reported and only fail under `--strict`.
//! A wrong act (expected silence, but a walk or an ask) fails both referees.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use anyhow::{bail, Result};

use crate::catalog::Catalog;
use crate::eval_on;
use crate::types::{Golden, RefereeKind};

#[derive(Clone, Debug)]
pub struct Case {
    pub id: String,
    pub snap: String,
    pub utterance: String,
    pub expect_page: Option<String>,
    pub expect_slots: BTreeMap<String, String>,
    pub expect_walk_substr: Option<String>,
    pub tags: Vec<String>,
    pub notes: String,
}

impl Case {
    pub fn lexical_must(&self) -> bool {
        !self.tags.iter().any(|t| t == "paraphrase")
    }
}

#[derive(Clone, Debug)]
pub struct CaseResult {
    pub id: String,
    pub snap: String,
    pub utterance: String,
    pub referee: RefereeKind,
    pub expected: Option<String>,
    pub got: Option<String>,
    pub slots_ok: bool,
    pub walk_ok: bool,
    pub wrong_act: bool,
    pub pass: bool,
    pub walk: Option<String>,
    pub reason: String,
    pub tags: Vec<String>,
}

pub fn all_cases(cat: &Catalog) -> Vec<Case> {
    let mut out = Vec::new();
    let mut seen = BTreeSet::new();
    for g in &cat.goldens {
        let c = from_golden(g);
        seen.insert(c.id.clone());
        out.push(c);
    }
    for c in authored() {
        if seen.insert(c.id.clone()) {
            out.push(c);
        }
    }
    out
}

fn from_golden(g: &Golden) -> Case {
    Case {
        id: g.id.clone(),
        snap: g.snap.clone(),
        utterance: g.utterance.clone(),
        expect_page: g.expect_page.clone(),
        expect_slots: g.expect_slots.clone().unwrap_or_default(),
        expect_walk_substr: None,
        tags: vec!["golden".into()],
        notes: g.notes.clone(),
    }
}

#[allow(clippy::too_many_arguments)]
fn c(
    id: &str,
    snap: &str,
    utterance: &str,
    page: Option<&str>,
    slots: &[(&str, &str)],
    walk: Option<&str>,
    tags: &[&str],
    notes: &str,
) -> Case {
    Case {
        id: id.into(),
        snap: snap.into(),
        utterance: utterance.into(),
        expect_page: page.map(str::to_string),
        expect_slots: slots
            .iter()
            .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
            .collect(),
        expect_walk_substr: walk.map(str::to_string),
        tags: tags.iter().map(|s| (*s).to_string()).collect(),
        notes: notes.into(),
    }
}

fn authored() -> Vec<Case> {
    let mut v = Vec::new();
    v.extend(wm());
    v.extend(audio());
    v.extend(launch());
    v.extend(mail_cal());
    v.extend(media_bucky());
    v.extend(house());
    v.extend(session_net());
    v.extend(refuses());
    v.extend(policy());
    v.extend(compound());
    v.extend(paraphrases());
    v.extend(browser());
    v
}

fn wm() -> Vec<Case> {
    vec![
        c(
            "wm-focus-firefox",
            "desk",
            "switch to firefox",
            Some("wm.focus"),
            &[("target", "firefox")],
            Some("focuswindow"),
            &["wm", "exact"],
            "Desk client.",
        ),
        c(
            "wm-focus-kitty",
            "desk",
            "focus kitty",
            Some("wm.focus"),
            &[("target", "kitty")],
            Some("focuswindow"),
            &["wm", "exact"],
            "Terminal class on fixture.",
        ),
        c(
            "wm-focus-code",
            "desk",
            "go to code",
            Some("wm.focus"),
            &[("target", "code")],
            Some("focuswindow"),
            &["wm", "exact"],
            "Editor.",
        ),
        c(
            "wm-fullscreen",
            "desk",
            "fullscreen",
            Some("wm.fullscreen"),
            &[],
            Some("fullscreen"),
            &["wm", "exact"],
            "Active client.",
        ),
        c(
            "wm-fullscreen-this",
            "desk",
            "fullscreen this",
            Some("wm.fullscreen"),
            &[("target", "active")],
            Some("fullscreen"),
            &["wm", "exact"],
            "Named this.",
        ),
        c(
            "wm-ws-1",
            "desk",
            "workspace one",
            Some("wm.workspace"),
            &[("ws", "1")],
            Some("workspace 1"),
            &["wm", "exact"],
            "",
        ),
        c(
            "wm-ws-3",
            "desk",
            "go to three",
            Some("wm.workspace"),
            &[("ws", "3")],
            Some("workspace 3"),
            &["wm", "exact"],
            "",
        ),
        c(
            "wm-ws-special",
            "desk",
            "special workspace",
            Some("wm.workspace"),
            &[("ws", "special")],
            Some("workspace special"),
            &["wm", "exact"],
            "Scratch.",
        ),
        c(
            "wm-move-1",
            "desk",
            "put this on one",
            Some("wm.move_ws"),
            &[("ws", "1")],
            Some("movetoworkspace"),
            &["wm", "exact"],
            "",
        ),
        c(
            "wm-close",
            "desk",
            "close this",
            Some("wm.close"),
            &[("target", "active")],
            Some("killactive"),
            &["wm", "confirm"],
            "Confirm policy.",
        ),
        c(
            "wm-close-browser",
            "desk",
            "kill the other browser",
            Some("wm.close"),
            &[("target", "firefox")],
            Some("killactive"),
            &["wm", "confirm"],
            "",
        ),
        c(
            "wm-float",
            "desk",
            "float this",
            Some("wm.float"),
            &[],
            Some("togglefloating"),
            &["wm", "exact"],
            "",
        ),
        c(
            "wm-tile",
            "desk",
            "tile this",
            Some("wm.float"),
            &[],
            Some("togglefloating"),
            &["wm", "exact"],
            "Alias of float toggle.",
        ),
        c(
            "wm-monitor-dead",
            "desk",
            "other monitor",
            None,
            &[],
            None,
            &["wm", "dead"],
            "Single output fixture.",
        ),
        c(
            "wm-split-term",
            "desk",
            "split with terminal",
            Some("wm.split_beside"),
            &[("app", "kitty")],
            Some("exec"),
            &["wm", "exact"],
            "Allowlisted split.",
        ),
        c(
            "wm-ask-focused",
            "desk",
            "what's focused",
            Some("wm.ask_focused"),
            &[],
            None,
            &["wm", "ask"],
            "",
        ),
        c(
            "wm-ask-where-ff",
            "desk",
            "which workspace is firefox on",
            Some("wm.ask_where"),
            &[("target", "firefox")],
            None,
            &["wm", "ask"],
            "",
        ),
        c(
            "wm-cycle",
            "desk",
            "cycle windows",
            Some("wm.cycle_next"),
            &[],
            Some("cyclenext"),
            &["wm", "exact"],
            "",
        ),
        c(
            "wm-other-window",
            "desk",
            "other window",
            Some("wm.cycle_next"),
            &[],
            Some("cyclenext"),
            &["wm", "exact"],
            "",
        ),
        c(
            "wm-focus-right",
            "desk",
            "focus right",
            Some("wm.movefocus"),
            &[("dir", "right")],
            Some("movefocus r"),
            &["wm", "exact"],
            "",
        ),
        c(
            "wm-focus-up",
            "desk",
            "focus up",
            Some("wm.movefocus"),
            &[("dir", "up")],
            Some("movefocus u"),
            &["wm", "exact"],
            "",
        ),
        c(
            "wm-focus-down",
            "desk",
            "move focus down",
            Some("wm.movefocus"),
            &[("dir", "down")],
            Some("movefocus d"),
            &["wm", "exact"],
            "",
        ),
        c(
            "wm-pin-window",
            "desk",
            "pin the window",
            Some("wm.pin"),
            &[],
            Some("pin"),
            &["wm", "exact"],
            "",
        ),
        c(
            "wm-always-on-top",
            "desk",
            "always on top",
            Some("wm.pin"),
            &[],
            Some("pin"),
            &["wm", "exact"],
            "",
        ),
        c(
            "wm-center-window",
            "desk",
            "center the window",
            Some("wm.center"),
            &[],
            Some("centerwindow"),
            &["wm", "exact"],
            "",
        ),
        c(
            "wm-swap-dead",
            "desk",
            "swap screens",
            None,
            &[],
            None,
            &["wm", "dead"],
            "One monitor.",
        ),
    ]
}

fn audio() -> Vec<Case> {
    vec![
        c(
            "audio-down-little",
            "desk",
            "turn it down a bit",
            Some("audio.bump"),
            &[("direction", "down"), ("amount", "little")],
            Some("5%-"),
            &["audio", "exact"],
            "",
        ),
        c(
            "audio-up-lot",
            "desk",
            "volume way up",
            Some("audio.bump"),
            &[("direction", "up"), ("amount", "lot")],
            Some("10%+"),
            &["audio", "exact"],
            "Lot notch.",
        ),
        c(
            "audio-quieter",
            "desk",
            "quieter",
            Some("audio.bump"),
            &[("direction", "down")],
            Some("%-"),
            &["audio", "exact"],
            "Direction alias.",
        ),
        c(
            "audio-louder",
            "desk",
            "louder",
            Some("audio.bump"),
            &[("direction", "up")],
            Some("%+"),
            &["audio", "exact"],
            "",
        ),
        c(
            "audio-lower",
            "desk",
            "lower the volume",
            Some("audio.bump"),
            &[("direction", "down")],
            Some("%-"),
            &["audio", "exact"],
            "",
        ),
        c(
            "audio-silence-alias",
            "desk",
            "silence",
            Some("audio.mute"),
            &[],
            Some("set-mute"),
            &["audio", "exact"],
            "",
        ),
        c(
            "audio-shut-up",
            "desk",
            "shut up",
            Some("audio.mute"),
            &[],
            Some("set-mute"),
            &["audio", "exact"],
            "",
        ),
        c(
            "audio-unmute-dead",
            "desk",
            "unmute",
            None,
            &[],
            None,
            &["audio", "dead"],
            "Desk is not muted.",
        ),
        c(
            "audio-speakers",
            "desk",
            "speakers",
            Some("audio.speakers"),
            &[],
            Some("set-default"),
            &["audio", "exact"],
            "",
        ),
        c(
            "audio-ask",
            "desk",
            "how loud am I",
            Some("audio.ask_level"),
            &[],
            None,
            &["audio", "ask"],
            "",
        ),
        c(
            "audio-ask-volume",
            "desk",
            "what's the volume",
            Some("audio.ask_level"),
            &[],
            None,
            &["audio", "ask"],
            "",
        ),
        c(
            "audio-mic-on-dead",
            "desk",
            "unmute the mic",
            None,
            &[],
            None,
            &["audio", "dead"],
            "Mic not muted on fixture.",
        ),
        c(
            "audio-tv-hdmi",
            "desk",
            "hdmi audio",
            Some("audio.tv"),
            &[],
            Some("hdmi"),
            &["audio", "exact"],
            "",
        ),
        c(
            "audio-tv-on-tv",
            "desk",
            "audio on the tv",
            Some("audio.tv"),
            &[],
            Some("hdmi"),
            &["audio", "exact"],
            "",
        ),
    ]
}

fn launch() -> Vec<Case> {
    vec![
        c(
            "launch-firefox",
            "desk",
            "open firefox",
            Some("launch.app"),
            &[("app", "firefox")],
            Some("focuswindow"),
            &["launch", "exact"],
            "Already running → focus.",
        ),
        c(
            "launch-files",
            "desk",
            "open files",
            Some("launch.app"),
            &[("app", "files")],
            Some("exec"),
            &["launch", "exact"],
            "Not running → exec.",
        ),
        c(
            "launch-thunderbird",
            "desk",
            "launch thunderbird",
            Some("launch.app"),
            &[("app", "thunderbird")],
            Some("exec"),
            &["launch", "exact"],
            "",
        ),
        c(
            "launch-ask-ff",
            "desk",
            "is firefox already open",
            Some("launch.ask_running"),
            &[("app", "firefox")],
            None,
            &["launch", "ask"],
            "Yes on desk.",
        ),
        c(
            "launch-ask-jellyfin-desk",
            "desk",
            "is jellyfin already open",
            Some("launch.ask_running"),
            &[("app", "jellyfin")],
            None,
            &["launch", "ask"],
            "No client.",
        ),
    ]
}

fn mail_cal() -> Vec<Case> {
    vec![
        c(
            "mail-unread",
            "desk",
            "any unread mail",
            Some("mail.unread"),
            &[],
            None,
            &["mail", "ask", "private"],
            "",
        ),
        c(
            "mail-unread-inbox",
            "desk",
            "what's in my inbox",
            Some("mail.unread"),
            &[],
            None,
            &["mail", "ask", "private"],
            "",
        ),
        c(
            "mail-school",
            "desk",
            "mail from school",
            Some("mail.from"),
            &[("who", "school")],
            None,
            &["mail", "ask", "private"],
            "",
        ),
        c(
            "mail-anyone",
            "desk",
            "any emails from anyone",
            Some("mail.from"),
            &[("who", "anyone")],
            None,
            &["mail", "ask", "private"],
            "",
        ),
        c(
            "mail-open-dave",
            "desk",
            "open Dave's last mail",
            Some("mail.open_last"),
            &[("who", "dave")],
            None,
            &["mail", "confirm", "private"],
            "Walk waits on yes.",
        ),
        c(
            "mail-flag",
            "desk",
            "flag that",
            Some("mail.flag"),
            &[],
            None,
            &["mail", "private"],
            "",
        ),
        c(
            "cal-today",
            "desk",
            "how busy am I today",
            Some("calendar.ask_today"),
            &[],
            None,
            &["calendar", "ask", "private"],
            "",
        ),
        c(
            "cal-whats-next",
            "desk",
            "what's next on my calendar",
            Some("calendar.ask_next"),
            &[],
            None,
            &["calendar", "ask", "private"],
            "",
        ),
        c(
            "notify-last",
            "desk",
            "what was that ping",
            Some("notify.read_last"),
            &[],
            None,
            &["notify", "private"],
            "History patched on desk.",
        ),
        c(
            "notify-last-notif",
            "desk",
            "last notification",
            Some("notify.read_last"),
            &[],
            None,
            &["notify", "private"],
            "",
        ),
        c(
            "notify-clear",
            "desk",
            "clear the ping",
            Some("notify.dismiss"),
            &[],
            Some("dismiss"),
            &["notify", "exact"],
            "",
        ),
    ]
}

fn media_bucky() -> Vec<Case> {
    vec![
        c(
            "media-resume-living",
            "guest-living",
            "put the show back on",
            Some("media.play_pause"),
            &[],
            Some("play-pause"),
            &["media", "exact"],
            "Alias without jellyfin target collision.",
        ),
        c(
            "media-play",
            "guest-living",
            "play",
            Some("media.play_pause"),
            &[],
            Some("play-pause"),
            &["media", "exact"],
            "",
        ),
        c(
            "media-next",
            "guest-living",
            "skip",
            Some("media.next"),
            &[],
            Some("playerctl next"),
            &["media", "exact"],
            "",
        ),
        c(
            "media-next-one",
            "guest-living",
            "next one",
            Some("media.next"),
            &[],
            Some("playerctl next"),
            &["media", "exact"],
            "",
        ),
        c(
            "media-prev",
            "guest-living",
            "go back",
            Some("media.prev"),
            &[],
            Some("previous"),
            &["media", "exact"],
            "",
        ),
        c(
            "media-now",
            "guest-living",
            "what's playing",
            Some("media.ask_now"),
            &[],
            None,
            &["media", "ask"],
            "",
        ),
        c(
            "media-dead-desk",
            "desk",
            "pause",
            None,
            &[],
            None,
            &["media", "dead"],
            "No player on desk.",
        ),
        c(
            "media-skip-desk-dead",
            "desk",
            "skip",
            None,
            &[],
            None,
            &["media", "dead"],
            "",
        ),
        c(
            "bucky-hide-done",
            "desk",
            "i'm done",
            Some("bucky.hide"),
            &[],
            Some("buckyboi"),
            &["bucky", "exact"],
            "",
        ),
        c(
            "bucky-go-away",
            "desk",
            "go away",
            Some("bucky.hide"),
            &[],
            Some("buckyboi"),
            &["bucky", "exact"],
            "",
        ),
        c(
            "bucky-listen",
            "desk",
            "listen",
            Some("bucky.listen"),
            &[],
            None,
            &["bucky", "exact"],
            "",
        ),
        c(
            "bucky-stop-listen",
            "desk",
            "stop listening",
            Some("bucky.mic_mute"),
            &[],
            None,
            &["bucky", "exact"],
            "ASR mute, not wpctl source.",
        ),
        c(
            "bucky-who",
            "desk",
            "who is here",
            Some("bucky.ask_who"),
            &[],
            None,
            &["bucky", "ask"],
            "",
        ),
        c(
            "bucky-who-talking",
            "desk",
            "who am i talking to",
            Some("bucky.ask_who"),
            &[],
            None,
            &["bucky", "ask", "paraphrase"],
            "Not an authored alias.",
        ),
    ]
}

fn house() -> Vec<Case> {
    vec![
        c(
            "scene-living-watch",
            "guest-living",
            "living watch",
            Some("scene.apply"),
            &[("scene", "living-watch")],
            Some("apply_scene"),
            &["scene", "exact"],
            "",
        ),
        c(
            "scene-quiet",
            "desk",
            "quiet hours",
            Some("scene.apply"),
            &[("scene", "quiet-hours")],
            Some("apply_scene"),
            &["scene", "exact"],
            "",
        ),
        c(
            "scene-bedside",
            "desk",
            "bedside night",
            Some("scene.apply"),
            &[("scene", "bedside-night")],
            Some("apply_scene"),
            &["scene", "exact"],
            "",
        ),
        c(
            "scene-house-idle",
            "desk",
            "house idle",
            Some("scene.apply"),
            &[("scene", "house-idle")],
            Some("apply_scene"),
            &["scene", "exact"],
            "",
        ),
        c(
            "scene-ask",
            "kitchen",
            "what's the scene in the kitchen",
            Some("scene.ask_active"),
            &[("room", "kitchen")],
            None,
            &["scene", "ask"],
            "",
        ),
        c(
            "scene-handoff-deny",
            "desk",
            "put it on the tv",
            None,
            &[],
            None,
            &["scene", "dead", "reserved"],
            "Authored and denied.",
        ),
        c(
            "scene-handoff-living",
            "guest-living",
            "put it on the living room",
            None,
            &[],
            None,
            &["scene", "dead", "reserved"],
            "",
        ),
        c(
            "lights-down",
            "desk",
            "lights down",
            Some("lights.bump"),
            &[("direction", "down")],
            Some("brightness_step"),
            &["lights", "exact"],
            "",
        ),
        c(
            "lights-dim",
            "desk",
            "dim the lights",
            Some("lights.bump"),
            &[("direction", "down")],
            Some("brightness_step"),
            &["lights", "exact"],
            "",
        ),
        c(
            "lights-brighter",
            "desk",
            "brighter",
            Some("lights.bump"),
            &[("direction", "up")],
            Some("brightness_step"),
            &["lights", "exact"],
            "",
        ),
        c(
            "lights-off-here",
            "desk",
            "lights off",
            Some("lights.off"),
            &[],
            Some("turn_off"),
            &["lights", "exact"],
            "",
        ),
        c(
            "timer-start-10",
            "desk",
            "timer ten minutes",
            Some("timer.start"),
            &[("mins", "10")],
            Some("10m"),
            &["timer", "exact"],
            "No running timer — start is live.",
        ),
        c(
            "timer-start-15",
            "desk",
            "start a fifteen minute timer",
            Some("timer.start"),
            &[("mins", "15")],
            Some("15m"),
            &["timer", "exact"],
            "",
        ),
        c(
            "timer-start-20",
            "kitchen",
            "timer twenty minutes",
            Some("timer.start"),
            &[("mins", "20")],
            Some("20m"),
            &["timer", "exact"],
            "",
        ),
        c(
            "timer-cancel-kit",
            "kitchen",
            "cancel the timer",
            Some("timer.cancel"),
            &[],
            Some("cancel"),
            &["timer", "exact"],
            "",
        ),
        c(
            "timer-cancel-desk-dead",
            "desk",
            "cancel the timer",
            None,
            &[],
            None,
            &["timer", "dead"],
            "",
        ),
        c(
            "timer-ask-desk-dead",
            "desk",
            "how long on the oven",
            None,
            &[],
            None,
            &["timer", "dead"],
            "No timer on desk.",
        ),
        c(
            "display-up",
            "desk",
            "screen brighter",
            Some("display.bump"),
            &[("direction", "up")],
            Some("brightnessctl"),
            &["display", "exact"],
            "",
        ),
        c(
            "display-brighter-panel",
            "desk",
            "screen brighter",
            Some("display.bump"),
            &[("direction", "up")],
            Some("brightnessctl"),
            &["display", "exact"],
            "",
        ),
        c(
            "climate-colder",
            "kitchen",
            "colder",
            Some("climate.bump"),
            &[("direction", "down")],
            Some("step"),
            &["climate", "exact"],
            "",
        ),
        c(
            "climate-cooler",
            "kitchen",
            "a bit cooler",
            Some("climate.bump"),
            &[("direction", "down"), ("amount", "little")],
            Some("step"),
            &["climate", "exact"],
            "",
        ),
        c(
            "climate-heater-off",
            "kitchen",
            "heater off",
            Some("climate.off"),
            &[],
            Some("turn_off"),
            &["climate", "exact"],
            "",
        ),
        c(
            "climate-ac-off",
            "kitchen",
            "ac off",
            Some("climate.off"),
            &[],
            Some("turn_off"),
            &["climate", "exact"],
            "",
        ),
        c(
            "capture-snap",
            "desk",
            "snap the screen",
            Some("capture.screenshot"),
            &[],
            Some("grim"),
            &["capture", "confirm"],
            "",
        ),
        c(
            "capture-grab",
            "desk",
            "grab the screen",
            Some("capture.screenshot"),
            &[],
            Some("grim"),
            &["capture", "confirm"],
            "",
        ),
        c(
            "capture-region-grab",
            "desk",
            "grab a region",
            Some("capture.region"),
            &[],
            Some("slurp"),
            &["capture", "confirm"],
            "",
        ),
        c(
            "weather-whats",
            "desk",
            "what's the weather",
            Some("weather.ask"),
            &[],
            None,
            &["weather", "ask"],
            "",
        ),
        c(
            "weather-outside",
            "desk",
            "weather outside",
            Some("weather.ask"),
            &[],
            None,
            &["weather", "ask"],
            "",
        ),
    ]
}

fn session_net() -> Vec<Case> {
    vec![
        c(
            "session-lock-box",
            "desk",
            "lock the box",
            Some("session.lock"),
            &[],
            Some("hyprlock"),
            &["session", "exact"],
            "",
        ),
        c(
            "session-lock-screen",
            "desk",
            "lock screen",
            Some("session.lock"),
            &[],
            Some("hyprlock"),
            &["session", "exact"],
            "",
        ),
        c(
            "session-screens-on",
            "desk",
            "screens on",
            Some("session.dpms"),
            &[("action", "on")],
            Some("dpms on"),
            &["session", "exact"],
            "",
        ),
        c(
            "session-sleep-displays",
            "desk",
            "sleep the displays",
            Some("session.dpms"),
            &[],
            Some("dpms"),
            &["session", "exact"],
            "",
        ),
        c(
            "net-status",
            "desk",
            "what's the network",
            Some("network.ask"),
            &[],
            None,
            &["network", "ask"],
            "",
        ),
        c(
            "net-online",
            "desk",
            "am i online",
            Some("network.ask"),
            &[],
            None,
            &["network", "ask"],
            "",
        ),
        c(
            "bt-status",
            "desk",
            "bluetooth status",
            Some("bluetooth.ask"),
            &[],
            None,
            &["bluetooth", "ask"],
            "",
        ),
    ]
}

fn refuses() -> Vec<Case> {
    let desk_none = |id, utt, notes| {
        c(
            id,
            "desk",
            utt,
            None,
            &[],
            None,
            &["refuse", "adversarial"],
            notes,
        )
    };
    vec![
        desk_none(
            "refuse-email-late",
            "email Dave that I'll be late",
            "Compose is not a page.",
        ),
        desk_none("refuse-send-mail", "send mail to Dave", "send regex."),
        desk_none("refuse-message-saying", "message Dave saying I'm sick", ""),
        desk_none("refuse-buy-tab", "buy the thing in that tab", ""),
        desk_none("refuse-purchase", "purchase that", ""),
        desk_none("refuse-click-tab", "click the second tab", ""),
        desk_none("refuse-click-pixel", "click the red button", ""),
        desk_none("refuse-invite", "invite Dave to standup", ""),
        desk_none("refuse-37pct", "set volume to 37 percent", ""),
        desk_none("refuse-40pct", "set the volume to 40%", ""),
        desk_none("refuse-22c", "set the thermostat to 22.7", ""),
        desk_none("refuse-kelvin", "set it to 3500 kelvin", ""),
        desk_none("refuse-cozy2", "make it look nicer", ""),
        desk_none("refuse-usual", "whatever I usually do", ""),
        desk_none("refuse-whole-thread", "read me the whole thread", ""),
        desk_none("refuse-random-site", "open a random website", ""),
        desk_none("refuse-curl", "run this curl", ""),
        desk_none("refuse-wake-seven", "wake me at seven", ""),
        desk_none("refuse-email-png", "email that screenshot", ""),
        desk_none("refuse-wifi", "turn off wifi", ""),
        desk_none("refuse-dns", "change dns", ""),
        desk_none("refuse-pair-bt", "pair the headphones", ""),
        desk_none("refuse-suspend", "suspend the computer", ""),
        desk_none("refuse-shutdown", "shut down", ""),
        desk_none("refuse-red-one", "the red one", ""),
        desk_none(
            "refuse-left-of-red",
            "the one on the left of the red one",
            "",
        ),
        desk_none(
            "refuse-close-everything",
            "close everything",
            "wm.close refuse list.",
        ),
        desk_none(
            "refuse-packing",
            "should I take a jacket",
            "weather.ask refuse.",
        ),
        desk_none("refuse-free-bin", "open a random website", ""),
        c(
            "refuse-send-guest",
            "guest-living",
            "email Dave that I'll be late",
            None,
            &[],
            None,
            &["refuse", "guest"],
            "Still refuse when guest.",
        ),
        c(
            "refuse-percent-kitchen",
            "kitchen",
            "set volume to 37 percent",
            None,
            &[],
            None,
            &["refuse"],
            "",
        ),
        c(
            "refuse-handoff-put-tv",
            "kitchen",
            "put this on the TV",
            None,
            &[],
            None,
            &["refuse", "scene"],
            "Handoff reserved — may also miss aliases.",
        ),
    ]
}

fn policy() -> Vec<Case> {
    vec![
        c(
            "policy-mail-guest",
            "guest-living",
            "any unread mail",
            None,
            &[],
            None,
            &["policy", "guest", "mail"],
            "Private/mail gone.",
        ),
        c(
            "policy-inbox-guest",
            "guest-living",
            "what's in my inbox",
            None,
            &[],
            None,
            &["policy", "guest", "mail"],
            "",
        ),
        c(
            "policy-cal-guest",
            "guest-living",
            "how busy am I today",
            None,
            &[],
            None,
            &["policy", "guest", "calendar"],
            "",
        ),
        c(
            "policy-cal-next-guest2",
            "guest-living",
            "what's next on my calendar",
            None,
            &[],
            None,
            &["policy", "guest", "calendar"],
            "",
        ),
        c(
            "policy-notify-guest",
            "guest-living",
            "last notification",
            None,
            &[],
            None,
            &["policy", "guest", "notify"],
            "notify.read_last is private.",
        ),
        c(
            "policy-volume-guest",
            "guest-living",
            "mute",
            Some("audio.mute"),
            &[],
            Some("set-mute"),
            &["policy", "guest", "household"],
            "Household still live.",
        ),
        c(
            "policy-pause-guest",
            "guest-living",
            "pause",
            Some("media.play_pause"),
            &[],
            Some("play-pause"),
            &["policy", "guest", "household"],
            "",
        ),
        c(
            "policy-lock-guest",
            "guest-living",
            "lock this",
            None,
            &[],
            None,
            &["policy", "guest"],
            "Guest is not the owner.",
        ),
        c(
            "policy-someone-coming",
            "guest-living",
            "someone's coming",
            None,
            &[],
            None,
            &["policy", "guest"],
            "Guest is not the owner.",
        ),
        c(
            "policy-shared-only",
            "guest-living",
            "shared only",
            None,
            &[],
            None,
            &["policy", "guest"],
            "Guest is not the owner.",
        ),
        c(
            "policy-guest-mode",
            "guest-living",
            "guest mode",
            None,
            &[],
            None,
            &["policy", "guest"],
            "Guest is not the owner.",
        ),
    ]
}

fn compound() -> Vec<Case> {
    vec![
        c(
            "cmp-jf-fs-desk",
            "desk",
            "switch to jellyfin and fullscreen",
            Some("launch.app"),
            &[("app", "jellyfin")],
            Some("exec"),
            &["compound", "wm"],
            "First take only; no client → launch.",
        ),
        c(
            "cmp-mute-hide",
            "desk",
            "mute and hide the buddy",
            Some("audio.mute"),
            &[],
            Some("set-mute"),
            &["compound"],
            "First take mute.",
        ),
        c(
            "cmp-and-then-ws",
            "desk",
            "workspace two and then fullscreen",
            Some("wm.workspace"),
            &[("ws", "2")],
            Some("workspace 2"),
            &["compound"],
            "",
        ),
        c(
            "cmp-vol-headphones",
            "desk",
            "increase the volume a bit and headphones",
            Some("audio.bump"),
            &[("direction", "up")],
            Some("%+"),
            &["compound", "audio"],
            "",
        ),
    ]
}

fn paraphrases() -> Vec<Case> {
    vec![
        c(
            "p-vol-up",
            "desk",
            "turn it up a little",
            Some("audio.bump"),
            &[("direction", "up")],
            Some("%+"),
            &["paraphrase", "audio"],
            "L2-style.",
        ),
        c(
            "p-vol-louder-please",
            "desk",
            "a bit louder please",
            Some("audio.bump"),
            &[("direction", "up"), ("amount", "little")],
            Some("%+"),
            &["paraphrase", "audio"],
            "",
        ),
        c(
            "p-mute-speakers",
            "desk",
            "silence the speakers",
            Some("audio.mute"),
            &[],
            Some("set-mute"),
            &["paraphrase", "audio"],
            "May collide with speakers sink.",
        ),
        c(
            "p-lock",
            "desk",
            "lock the workstation",
            Some("session.lock"),
            &[],
            Some("hyprlock"),
            &["paraphrase", "session"],
            "",
        ),
        c(
            "p-dim-panel",
            "desk",
            "make the screen a bit darker",
            Some("display.bump"),
            &[("direction", "down")],
            Some("brightnessctl"),
            &["paraphrase", "display"],
            "",
        ),
        c(
            "p-open-bot",
            "desk",
            "launch the bot",
            Some("launch.app"),
            &[("app", "grokbot")],
            Some("exec"),
            &["paraphrase", "launch"],
            "",
        ),
        c(
            "p-mail-dave",
            "desk",
            "anything from Dave in my inbox",
            Some("mail.from"),
            &[("who", "dave")],
            None,
            &["paraphrase", "mail"],
            "",
        ),
        c(
            "p-pause-show",
            "guest-living",
            "pause the show",
            Some("media.play_pause"),
            &[],
            Some("play-pause"),
            &["paraphrase", "media"],
            "Also an exact alias path.",
        ),
        c(
            "p-rain",
            "desk",
            "will it rain",
            Some("weather.ask"),
            &[],
            None,
            &["paraphrase", "weather"],
            "",
        ),
        c(
            "p-focus-browser",
            "desk",
            "show me firefox",
            Some("wm.focus"),
            &[("target", "firefox")],
            Some("focuswindow"),
            &["paraphrase", "wm"],
            "",
        ),
        c(
            "p-next-ws",
            "desk",
            "jump to workspace two",
            Some("wm.workspace"),
            &[("ws", "2")],
            Some("workspace 2"),
            &["paraphrase", "wm"],
            "",
        ),
        c(
            "p-fs-jellyfin",
            "desk-jellyfin",
            "make jellyfin full screen",
            Some("wm.fullscreen"),
            &[("target", "jellyfin")],
            Some("fullscreen"),
            &["paraphrase", "wm"],
            "",
        ),
        c(
            "p-timer-oven",
            "kitchen",
            "how much time left on the oven",
            Some("timer.ask"),
            &[],
            None,
            &["paraphrase", "timer"],
            "",
        ),
        c(
            "p-cook",
            "kitchen",
            "start cooking in the kitchen",
            Some("scene.apply"),
            &[("scene", "kitchen-cook")],
            Some("apply_scene"),
            &["paraphrase", "scene"],
            "",
        ),
        c(
            "p-night",
            "desk",
            "turn on night light",
            Some("display.night"),
            &[],
            Some("hyprsunset"),
            &["paraphrase", "display"],
            "",
        ),
        c(
            "p-screenshot",
            "desk",
            "take a screenshot",
            Some("capture.screenshot"),
            &[],
            Some("grim"),
            &["paraphrase", "capture"],
            "",
        ),
        c(
            "p-online",
            "desk",
            "are we connected",
            Some("network.ask"),
            &[],
            None,
            &["paraphrase", "network"],
            "",
        ),
        c(
            "p-bt",
            "desk",
            "is bluetooth powered",
            Some("bluetooth.ask"),
            &[],
            None,
            &["paraphrase", "bluetooth"],
            "",
        ),
        c(
            "p-hide-buddy",
            "desk",
            "put the buddy away",
            Some("bucky.hide"),
            &[],
            Some("buckyboi"),
            &["paraphrase", "bucky"],
            "",
        ),
        c(
            "p-refuse-late",
            "desk",
            "send Dave a message that I'm running late",
            None,
            &[],
            None,
            &["paraphrase", "refuse"],
            "Must still silence.",
        ),
        c(
            "p-refuse-pct",
            "desk",
            "crank the volume to 40%",
            None,
            &[],
            None,
            &["paraphrase", "refuse"],
            "",
        ),
    ]
}

fn browser() -> Vec<Case> {
    vec![
        c(
            "browser-tab-new",
            "desk-zen",
            "new tab",
            Some("browser.tab_new"),
            &[("app", "zen")],
            Some("key = \"T\""),
            &["browser", "exact"],
            "Ctrl+T on the focused Zen window.",
        ),
        c(
            "browser-tab-close",
            "desk-zen",
            "close tab",
            Some("browser.tab_close"),
            &[("app", "zen")],
            Some("key = \"W\""),
            &["browser", "exact", "confirm"],
            "Not wm.close.",
        ),
        c(
            "browser-tab-reopen",
            "desk-zen",
            "reopen tab",
            Some("browser.tab_reopen"),
            &[("app", "zen")],
            Some("CTRL + SHIFT"),
            &["browser", "exact"],
            "Ctrl+Shift+T.",
        ),
        c(
            "browser-tab-next",
            "desk-zen",
            "next tab",
            Some("browser.tab_next"),
            &[("app", "zen")],
            Some("key = \"Tab\""),
            &["browser", "exact"],
            "Live door beats a dead same-length alias.",
        ),
        c(
            "browser-tab-prev",
            "desk-zen",
            "previous tab",
            Some("browser.tab_prev"),
            &[("app", "zen")],
            Some("CTRL + SHIFT"),
            &["browser", "exact"],
            "",
        ),
        c(
            "browser-back",
            "desk-zen",
            "browser back",
            Some("browser.back"),
            &[("app", "zen")],
            Some("key = \"Left\""),
            &["browser", "exact"],
            "Not media.prev.",
        ),
        c(
            "browser-forward",
            "desk-zen",
            "browser forward",
            Some("browser.forward"),
            &[("app", "zen")],
            Some("key = \"Right\""),
            &["browser", "exact"],
            "",
        ),
        c(
            "browser-reload",
            "desk-zen",
            "reload the page",
            Some("browser.reload"),
            &[("app", "zen")],
            Some("mods = \"CTRL\", key = \"R\""),
            &["browser", "exact"],
            "",
        ),
        c(
            "browser-reload-hard",
            "desk-zen",
            "hard reload",
            Some("browser.reload_hard"),
            &[("app", "zen")],
            Some("CTRL + SHIFT"),
            &["browser", "exact"],
            "",
        ),
        c(
            "browser-home",
            "desk-zen",
            "browser home",
            Some("browser.home"),
            &[("app", "zen")],
            Some("key = \"Home\""),
            &["browser", "exact"],
            "",
        ),
        c(
            "browser-find",
            "desk-zen",
            "find in page",
            Some("browser.find"),
            &[("app", "zen")],
            Some("key = \"F\""),
            &["browser", "exact"],
            "Bar only, no query.",
        ),
        c(
            "browser-zoom-little",
            "desk-zen",
            "zoom in",
            Some("browser.zoom"),
            &[("app", "zen"), ("direction", "up"), ("amount", "little")],
            Some("key = \"plus\""),
            &["browser", "exact"],
            "One chord.",
        ),
        c(
            "browser-zoom-lot",
            "desk-zen",
            "zoom in a lot",
            Some("browser.zoom"),
            &[("app", "zen"), ("direction", "up"), ("amount", "lot")],
            Some("key = \"plus\""),
            &["browser", "exact"],
            "Three identical chords.",
        ),
        c(
            "browser-zoom-reset",
            "desk-zen",
            "actual size",
            Some("browser.zoom_reset"),
            &[("app", "zen")],
            Some("key = \"0\""),
            &["browser", "exact"],
            "",
        ),
        c(
            "browser-fullscreen-page",
            "desk-zen",
            "fullscreen the page",
            Some("browser.fullscreen"),
            &[("app", "zen")],
            Some("mods = \"\", key = \"F11\""),
            &["browser", "exact"],
            "Empty mods, not wm.fullscreen.",
        ),
        c(
            "browser-focus-mapped",
            "desk-zen",
            "focus the browser",
            Some("browser.focus"),
            &[("app", "zen")],
            Some("hl.dsp.focus"),
            &["browser", "exact"],
            "Address only.",
        ),
        c(
            "browser-focus-exec",
            "desk-zen",
            "firefox window",
            Some("browser.focus"),
            &[("app", "firefox")],
            Some("exec_cmd(\"firefox\")"),
            &["browser", "exact"],
            "Unmapped, allowlisted, bins has the binary.",
        ),
        c(
            "browser-private-ff",
            "desk",
            "private window",
            Some("browser.private"),
            &[("app", "firefox")],
            Some("CTRL + SHIFT\", key = \"P\""),
            &["browser", "exact"],
            "Firefox only.",
        ),
        c(
            "browser-devtools",
            "desk",
            "dev tools",
            Some("browser.dev_tools"),
            &[("app", "firefox")],
            Some("key = \"I\""),
            &["browser", "exact", "confirm"],
            "Owner.",
        ),
        c(
            "browser-ask-open",
            "desk-zen",
            "is the browser open",
            Some("browser.ask_open"),
            &[("app", "zen")],
            None,
            &["browser", "ask"],
            "Allowlist, mapped or not.",
        ),
        c(
            "browser-downloads",
            "desk-zen",
            "how many downloads",
            Some("browser.downloads"),
            &[("app", "zen")],
            None,
            &["browser", "ask", "private"],
            "Bucket, not {ok:true}.",
        ),
        c(
            "browser-kitchen-close-tab",
            "kitchen",
            "close tab",
            Some("browser.tab_close"),
            &[("app", "chromium")],
            Some("send_shortcut"),
            &["browser", "exact"],
            "Chromium class matches.",
        ),
        c(
            "browser-close-tab-guest",
            "guest-living",
            "close tab",
            None,
            &[],
            None,
            &["browser", "dead"],
            "Dead alias longer than wm.close.",
        ),
        c(
            "browser-next-tab-guest",
            "guest-living",
            "next tab",
            None,
            &[],
            None,
            &["browser", "dead"],
            "Does not walk media.next.",
        ),
        c(
            "browser-fs-page-guest",
            "guest-living",
            "fullscreen the page",
            None,
            &[],
            None,
            &["browser", "dead"],
            "Does not walk wm.fullscreen.",
        ),
        c(
            "browser-close-this-guest",
            "guest-living",
            "close this",
            Some("wm.close"),
            &[("target", "active")],
            Some("killactive"),
            &["wm", "confirm"],
            "Bare close this stays.",
        ),
        c(
            "browser-close-this-kitchen",
            "kitchen",
            "close this",
            Some("wm.close"),
            &[("target", "active")],
            Some("killactive"),
            &["wm", "confirm"],
            "Bare close this stays.",
        ),
        c(
            "browser-next-bare",
            "guest-living",
            "next",
            Some("media.next"),
            &[],
            Some("playerctl next"),
            &["media", "exact"],
            "Bare next stays.",
        ),
        c(
            "browser-fullscreen-bare-zen",
            "desk-zen",
            "fullscreen",
            Some("wm.fullscreen"),
            &[],
            Some("fullscreen"),
            &["wm", "exact"],
            "Bare fullscreen stays wm.",
        ),
        c(
            "browser-close-bare-zen",
            "desk-zen",
            "close this",
            Some("wm.close"),
            &[("target", "active")],
            Some("killactive"),
            &["wm", "confirm"],
            "",
        ),
        c(
            "browser-private-chrome-dead",
            "desk-browsers",
            "chrome private window",
            None,
            &[],
            None,
            &["browser", "dead"],
            "Chrome private is reserved.",
        ),
        c(
            "browser-devtools-guest",
            "guest-living",
            "dev tools",
            None,
            &[],
            None,
            &["browser", "dead"],
            "Owner page, guest.",
        ),
        c(
            "browser-downloads-guest",
            "guest-living",
            "how many downloads",
            None,
            &[],
            None,
            &["browser", "dead", "private"],
            "Private.",
        ),
        c(
            "browser-pin-dead",
            "desk-zen",
            "pin tab",
            None,
            &[],
            None,
            &["browser", "dead"],
            "No builtin chord.",
        ),
        c(
            "browser-mute-tab-dead",
            "desk-zen",
            "mute tab",
            None,
            &[],
            None,
            &["browser", "dead"],
            "Does not mute the sink.",
        ),
        c(
            "browser-reader-dead",
            "desk-zen",
            "reader mode",
            None,
            &[],
            None,
            &["browser", "dead"],
            "",
        ),
        c(
            "browser-pip-dead",
            "desk-zen",
            "picture in picture",
            None,
            &[],
            None,
            &["browser", "dead"],
            "",
        ),
        c(
            "browser-bookmark-dead",
            "desk-zen",
            "bookmark this page",
            None,
            &[],
            None,
            &["browser", "dead"],
            "Folder list does not arm the chord.",
        ),
        c(
            "browser-translate-dead",
            "desk-zen",
            "translate this page",
            None,
            &[],
            None,
            &["browser", "dead"],
            "",
        ),
        c(
            "browser-download-cancel-dead",
            "desk-zen",
            "cancel the download",
            None,
            &[],
            None,
            &["browser", "dead"],
            "",
        ),
    ]
}

pub fn run_case(cat: &Catalog, case: &Case, referee: RefereeKind) -> CaseResult {
    let Some(snap) = cat.snap(&case.snap) else {
        return CaseResult {
            id: case.id.clone(),
            snap: case.snap.clone(),
            utterance: case.utterance.clone(),
            referee,
            expected: case.expect_page.clone(),
            got: Some("missing-snap".into()),
            slots_ok: false,
            walk_ok: false,
            wrong_act: false,
            pass: false,
            walk: None,
            reason: "snap not found".into(),
            tags: case.tags.clone(),
        };
    };
    match eval_on(cat, &case.utterance, snap, referee) {
        Err(e) => CaseResult {
            id: case.id.clone(),
            snap: case.snap.clone(),
            utterance: case.utterance.clone(),
            referee,
            expected: case.expect_page.clone(),
            got: None,
            slots_ok: false,
            walk_ok: false,
            wrong_act: false,
            pass: false,
            walk: None,
            reason: e.to_string(),
            tags: case.tags.clone(),
        },
        Ok(r) => {
            let got = r.take.page_id.clone();
            let page_ok = got == case.expect_page;
            let mut slots_ok = true;
            for (k, v) in &case.expect_slots {
                if r.take.slots.get(k) != Some(v) {
                    slots_ok = false;
                }
            }
            let walk = r.walk.as_ref().map(|w| w.command.clone());
            let walk_ok = match &case.expect_walk_substr {
                None => true,
                Some(sub) => walk.as_deref().is_some_and(|w| w.contains(sub)),
            };
            let wrong_act = case.expect_page.is_none()
                && got.is_some()
                && (r.walk.is_some() || r.answer.is_some());
            CaseResult {
                id: case.id.clone(),
                snap: case.snap.clone(),
                utterance: case.utterance.clone(),
                referee,
                expected: case.expect_page.clone(),
                got,
                slots_ok,
                walk_ok,
                wrong_act,
                pass: page_ok && slots_ok && walk_ok,
                walk,
                reason: r.take.reason.clone(),
                tags: case.tags.clone(),
            }
        }
    }
}

pub struct SuiteReport {
    pub results: Vec<CaseResult>,
    pub must_fail: usize,
    pub soft_fail: usize,
    pub wrong_acts: usize,
    /// Laya referee, server down. Cases were not run (no per-case client timeout).
    pub skipped: bool,
}

pub fn run_suite(cat: &Catalog, referee: RefereeKind, tag: Option<&str>) -> Result<SuiteReport> {
    let cases: Vec<Case> = all_cases(cat)
        .into_iter()
        .filter(|c| tag.is_none_or(|t| c.tags.iter().any(|x| x == t)))
        .collect();
    if cases.is_empty() {
        match tag {
            Some(t) => bail!("{t} tag matched nothing"),
            None => bail!("suite matched nothing"),
        }
    }
    if referee == RefereeKind::Laya && !crate::model::available(RefereeKind::Laya) {
        return Ok(SuiteReport {
            results: Vec::new(),
            must_fail: 0,
            soft_fail: 0,
            wrong_acts: 0,
            skipped: true,
        });
    }
    let mut results = Vec::with_capacity(cases.len());
    let mut must_fail = 0;
    let mut soft_fail = 0;
    let mut wrong_acts = 0;
    for case in &cases {
        let r = run_case(cat, case, referee);
        if r.wrong_act {
            wrong_acts += 1;
        }
        if !r.pass {
            if case.lexical_must() || referee != RefereeKind::Lexical {
                // paraphrase failures on lexical are soft; on laya, report as soft unless strict
                if case.lexical_must() && referee == RefereeKind::Lexical {
                    must_fail += 1;
                } else {
                    soft_fail += 1;
                }
            } else {
                soft_fail += 1;
            }
        }
        results.push(r);
    }
    Ok(SuiteReport {
        results,
        must_fail,
        soft_fail,
        wrong_acts,
        skipped: false,
    })
}

pub fn render_report(report: &SuiteReport, show_pass: bool) -> String {
    let mut s = String::new();
    let total = report.results.len();
    let pass = report.results.iter().filter(|r| r.pass).count();
    let _ = writeln!(
        s,
        "suite  {pass}/{total} pass  must_fail={}  paraphrase_or_laya_miss={}  wrong_act={}",
        report.must_fail, report.soft_fail, report.wrong_acts
    );
    let mut by_tag: BTreeMap<String, (u32, u32)> = BTreeMap::new();
    for r in &report.results {
        for t in &r.tags {
            let e = by_tag.entry(t.clone()).or_insert((0, 0));
            e.1 += 1;
            if r.pass {
                e.0 += 1;
            }
        }
    }
    let _ = writeln!(s, "by tag:");
    for (t, (ok, n)) in &by_tag {
        let _ = writeln!(s, "  {t:16} {ok}/{n}");
    }
    for r in &report.results {
        if r.pass && !show_pass {
            continue;
        }
        let mark = if r.pass {
            "✓"
        } else if r.wrong_act {
            "✗ACT"
        } else {
            "✗"
        };
        let _ = writeln!(
            s,
            "  {mark} {}  [{}] {:?} → {:?}  slots_ok={} walk_ok={}  {:?}",
            r.id, r.snap, r.utterance, r.got, r.slots_ok, r.walk_ok, r.expected
        );
        if !r.pass {
            if let Some(w) = &r.walk {
                let _ = writeln!(s, "      walk {w}");
            }
            if !r.reason.is_empty() {
                let _ = writeln!(s, "      {}", r.reason);
            }
        }
    }
    s
}

pub fn run_and_print(
    cat: &Catalog,
    referee: RefereeKind,
    tag: Option<&str>,
    show_pass: bool,
    strict: bool,
) -> Result<()> {
    println!("== {referee} ==");
    let report = run_suite(cat, referee, tag)?;
    if report.skipped {
        println!("Laya unavailable — skip");
        return Ok(());
    }
    print!("{}", render_report(&report, show_pass));
    if referee == RefereeKind::Lexical && report.must_fail > 0 {
        bail!("{} lexical must-pass cases failed", report.must_fail);
    }
    if report.wrong_acts > 0 {
        bail!("{} wrong acts on refuse/dead cases", report.wrong_acts);
    }
    if strict && !report.results.iter().all(|r| r.pass) {
        bail!(
            "strict: {} failing",
            report.results.iter().filter(|r| !r.pass).count()
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::Catalog;

    #[test]
    fn suite_has_breadth() {
        let cat = Catalog::load();
        let cases = all_cases(&cat);
        assert!(cases.len() >= 120, "got {}", cases.len());
        let tags: BTreeSet<_> = cases.iter().flat_map(|c| c.tags.iter().cloned()).collect();
        for need in ["wm", "audio", "refuse", "guest", "compound", "paraphrase"] {
            assert!(tags.iter().any(|t| t == need), "missing tag {need}");
        }
    }

    #[test]
    fn lexical_must_pass() {
        let cat = Catalog::load();
        let report = run_suite(&cat, RefereeKind::Lexical, None).expect("suite");
        if report.must_fail > 0 || report.wrong_acts > 0 {
            panic!("{}", render_report(&report, false));
        }
    }

    #[test]
    fn empty_tag_fails() {
        let cat = Catalog::load();
        let err = match run_suite(&cat, RefereeKind::Lexical, Some("holdout")) {
            Err(e) => e,
            Ok(_) => panic!("empty tag must fail"),
        };
        assert!(
            err.to_string().contains("holdout tag matched nothing"),
            "{err}"
        );
        let wm = run_suite(&cat, RefereeKind::Lexical, Some("wm")).expect("wm tag");
        assert!(!wm.skipped);
        assert!(!wm.results.is_empty());
    }

    #[test]
    fn browser_alias_lint() {
        let cat = Catalog::load();
        let cases = all_cases(&cat);
        let banned = [
            "next",
            "close",
            "zoom",
            "save",
            "back",
            "open",
            "focus",
            "mute",
            "play",
            "pause",
            "workspace",
        ];
        let mut errs = Vec::new();
        for page in cat.pages.iter().filter(|p| p.module == "browser") {
            for alias in &page.aliases {
                let n = crate::text::norm(alias);
                if n == "browser" {
                    errs.push(format!("{} alias is the bare word browser", page.id));
                }
                let raw: Vec<&str> = n.split_whitespace().collect();
                if raw.len() == 1 && banned.contains(&raw[0]) {
                    errs.push(format!("{} banned single-token alias {alias}", page.id));
                }
                if raw.len() < 2 {
                    continue;
                }
                let at = crate::text::tokens(alias);
                if at.is_empty() {
                    continue;
                }
                for case in &cases {
                    if !case.lexical_must() {
                        continue;
                    }
                    let Some(expect) = case.expect_page.as_deref() else {
                        continue;
                    };
                    if expect == page.id {
                        continue;
                    }
                    let ut = crate::text::tokens(&case.utterance);
                    if at.iter().all(|t| ut.iter().any(|u| u == t)) {
                        errs.push(format!(
                            "{} alias {alias:?} token-subsets {} {:?}",
                            page.id, case.id, case.utterance
                        ));
                    }
                }
            }
        }
        assert!(errs.is_empty(), "{}", errs.join("\n"));
    }
}
