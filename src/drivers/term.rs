//! Terminal doors. `term.focus` is the only exec.
//! `term.new_window` sends a chord to a terminal that is already mapped.
//! It never emits `hyprctl dispatch exec` (that is not `exec_cmd` on 0.56.2).

use std::collections::BTreeMap;

use serde_json::{json, Value};

use crate::classes::{bin_for_app, client_for_app};
use crate::keymap::{self, Chord};
use crate::types::{Page, Snap, WalkPlan};

pub fn is_live(page: &Page, snap: &Snap, slots: &BTreeMap<String, String>) -> (bool, String) {
    let Some(app) = slots.get("app") else {
        return (false, "app missing".into());
    };
    if !is_term_app(app) {
        return (false, "not a terminal".into());
    }
    match page.id.as_str() {
        "term.focus" => focus_live(snap, app),
        // New window included: unmapped is dead, not an exec.
        "term.new_window" | "term.new_tab" | "term.close" | "term.next" | "term.prev"
        | "term.font" => act_live(page, snap, app, slots),
        _ => (false, "unknown term page".into()),
    }
}

fn is_term_app(app: &str) -> bool {
    matches!(app, "foot" | "kitty" | "ghostty" | "alacritty" | "wezterm")
}

fn act_live(
    page: &Page,
    snap: &Snap,
    app: &str,
    slots: &BTreeMap<String, String>,
) -> (bool, String) {
    let Some(c) = client_for_app(snap, app) else {
        return (false, "no matching client".into());
    };
    if keymap::chord_for(&page.id, app, &snap.id, slots).is_none() {
        return (false, "reserved — no chord".into());
    }
    (true, format!("client {}", c.class))
}

fn focus_live(snap: &Snap, app: &str) -> (bool, String) {
    if client_for_app(snap, app).is_some() {
        return (true, format!("client {app}"));
    }
    let Some(bin) = bin_for_app(app) else {
        return (false, "terminal not mapped".into());
    };
    if snap.allowlist.iter().any(|a| a == app) && snap.bins.iter().any(|b| b == bin) {
        (true, format!("exec {bin}"))
    } else {
        (false, "terminal not mapped".into())
    }
}

pub fn fill_walk(page: &Page, slots: &BTreeMap<String, String>, snap: &Snap) -> WalkPlan {
    let app = slots.get("app").map(String::as_str).unwrap_or("");
    let driver = "term".to_string();
    if page.id == "term.focus" {
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
    let Some(chord) = keymap::chord_for(&page.id, app, &snap.id, slots) else {
        return WalkPlan {
            command: "reserved".into(),
            driver,
        };
    };
    let chord = shaped_chord(page, slots, chord);
    let reps = if page.id == "term.font" && slots.get("amount").map(String::as_str) == Some("lot") {
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

fn shaped_chord(page: &Page, slots: &BTreeMap<String, String>, mut chord: Chord) -> Chord {
    if page.id == "term.font" && slots.get("direction").map(String::as_str) == Some("down") {
        chord.key = "minus".into();
    }
    chord
}

pub fn fill_ask(page: &Page, _slots: &BTreeMap<String, String>, _snap: &Snap) -> Value {
    json!({ "unarmed": page.id })
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::Catalog;

    #[test]
    fn new_window_unmapped_is_reserved_not_exec() {
        let cat = Catalog::load();
        let snap = cat.snap("desk").unwrap();
        let page = cat.page("term.new_window").unwrap();
        let mut slots = BTreeMap::new();
        slots.insert("app".into(), "foot".into());
        let (ok, why) = is_live(page, snap, &slots);
        assert!(!ok, "{why}");
        let plan = fill_walk(page, &slots, snap);
        assert_eq!(plan.command, "reserved");
        assert!(!plan.command.contains("exec_cmd"), "{}", plan.command);
        assert!(!plan.command.contains("dispatch exec"), "{}", plan.command);
    }
}
