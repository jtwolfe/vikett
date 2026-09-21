//! Shared browser doors. Focus may exec; every other act needs a mapped client and a chord.

use std::collections::BTreeMap;

use serde_json::{json, Value};

use crate::classes::{bin_for_app, client_for_app};
use crate::keymap::{self, Chord};
use crate::types::{Page, Snap, WalkPlan};

pub fn is_live(page: &Page, snap: &Snap, slots: &BTreeMap<String, String>) -> (bool, String) {
    let Some(app) = slots.get("app") else {
        return (false, "app missing".into());
    };
    match page.id.as_str() {
        "browser.focus" => focus_live(snap, app),
        "browser.ask_open" => {
            if snap.allowlist.iter().any(|a| a == app) {
                (true, format!("allowlisted {app}"))
            } else {
                (false, "not allowlisted".into())
            }
        }
        "browser.downloads" => match snap.downloads {
            Some(n) => (true, format!("{n} downloads")),
            None => (false, "downloads unknown".into()),
        },
        _ => {
            let Some(c) = client_for_app(snap, app) else {
                return (false, "no matching client".into());
            };
            if keymap::chord_for(&page.id, app, snap).is_none() {
                return (false, "reserved — no chord".into());
            }
            (true, format!("client {}", c.class))
        }
    }
}

fn focus_live(snap: &Snap, app: &str) -> (bool, String) {
    if client_for_app(snap, app).is_some() {
        return (true, format!("client {app}"));
    }
    let Some(bin) = bin_for_app(app) else {
        return (false, "browser not mapped".into());
    };
    if snap.allowlist.iter().any(|a| a == app) && snap.bins.iter().any(|b| b == bin) {
        (true, format!("exec {bin}"))
    } else {
        (false, "browser not mapped".into())
    }
}

pub fn fill_walk(page: &Page, slots: &BTreeMap<String, String>, snap: &Snap) -> WalkPlan {
    let app = slots.get("app").map(String::as_str).unwrap_or("");
    let driver = "browser".to_string();
    if page.id == "browser.focus" {
        if let Some(c) = client_for_app(snap, app) {
            return WalkPlan {
                command: focus_cmd(&c.address),
                driver,
            };
        }
        let command = bin_for_app(app)
            .map(exec_cmd)
            .unwrap_or_else(|| "reserved".into());
        return WalkPlan { command, driver };
    }
    let Some(c) = client_for_app(snap, app) else {
        return WalkPlan {
            command: "reserved".into(),
            driver,
        };
    };
    let Some(chord) = keymap::chord_for(&page.id, app, snap) else {
        return WalkPlan {
            command: "reserved".into(),
            driver,
        };
    };
    let chord = zoom_key(page, slots, chord);
    let reps =
        if page.id == "browser.zoom" && slots.get("amount").map(String::as_str) == Some("lot") {
            3
        } else {
            1
        };
    let mut parts = vec![focus_cmd(&c.address)];
    for _ in 0..reps {
        parts.push(shortcut_cmd(&chord, &c.address));
    }
    WalkPlan {
        command: parts.join(" && "),
        driver,
    }
}

fn zoom_key(page: &Page, slots: &BTreeMap<String, String>, mut chord: Chord) -> Chord {
    if page.id == "browser.zoom" {
        chord.key = if slots.get("direction").map(String::as_str) == Some("down") {
            "minus".into()
        } else {
            "plus".into()
        };
    }
    chord
}

pub fn fill_ask(page: &Page, slots: &BTreeMap<String, String>, snap: &Snap) -> Value {
    let app = slots.get("app").cloned().unwrap_or_default();
    match page.id.as_str() {
        "browser.ask_open" => {
            let c = client_for_app(snap, &app);
            json!({
                "app": app,
                "open": c.is_some(),
                "focused": c.is_some_and(|c| c.focused),
            })
        }
        "browser.downloads" => {
            let Some(n) = snap.downloads else {
                return json!({ "unarmed": "browser.downloads" });
            };
            let bucket = if n == 0 {
                "none"
            } else if n <= 3 {
                "few"
            } else {
                "many"
            };
            json!({ "bucket": bucket })
        }
        _ => json!({ "unarmed": page.id }),
    }
}

fn window_sel(address: &str) -> String {
    if let Some(rest) = address.strip_prefix("address:") {
        format!("address:{rest}")
    } else {
        format!("address:{address}")
    }
}

fn focus_cmd(address: &str) -> String {
    let window = window_sel(address);
    format!("hyprctl dispatch 'hl.dsp.focus({{ window = \"{window}\" }})'")
}

fn shortcut_cmd(chord: &Chord, address: &str) -> String {
    let window = window_sel(address);
    format!(
        "hyprctl dispatch 'hl.dsp.send_shortcut({{ mods = \"{mods}\", key = \"{key}\", window = \"{window}\" }})'",
        mods = chord.mods,
        key = chord.key,
    )
}

fn exec_cmd(bin: &str) -> String {
    format!("hyprctl dispatch 'hl.dsp.exec_cmd(\"{bin}\")'")
}
