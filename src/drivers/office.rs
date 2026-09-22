//! LibreOffice. Save and next sheet send a chord when a client is mapped.
//! Zoom has no modifier chord, so it stays reserved. No cell address is a slot.
//! Nothing here execs.

use std::collections::BTreeMap;

use crate::classes::client_for_app;
use crate::drivers::hl;
use crate::keymap::{self, Chord};
use crate::types::{Page, Snap, WalkPlan};

pub fn is_live(page: &Page, snap: &Snap, slots: &BTreeMap<String, String>) -> (bool, String) {
    let Some(app) = slots.get("app").map(String::as_str) else {
        return (false, "app missing".into());
    };
    if app != "libreoffice" {
        return (false, "not libreoffice".into());
    }
    match page.id.as_str() {
        "office.save" | "office.zoom" | "office.next_sheet" => chord_live(page, snap, app, slots),
        _ => (false, "unknown office page".into()),
    }
}

pub fn fill_walk(page: &Page, slots: &BTreeMap<String, String>, snap: &Snap) -> WalkPlan {
    let app = slots.get("app").map(String::as_str).unwrap_or("");
    let Some(c) = client_for_app(snap, app) else {
        return reserved();
    };
    let Some(chord) = keymap::chord_for(&page.id, app, &snap.id, slots) else {
        return reserved();
    };
    let chord = shaped(&page.id, slots, chord);
    let reps = if page.id == "office.zoom" && slots.get("amount").map(String::as_str) == Some("lot")
    {
        3
    } else {
        1
    };
    repeat(&c.address, &chord, reps)
}

fn shaped(page_id: &str, slots: &BTreeMap<String, String>, mut chord: Chord) -> Chord {
    if page_id == "office.zoom" && slots.get("direction").map(String::as_str) == Some("down") {
        chord.key = "minus".into();
    }
    chord
}

fn chord_live(
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

fn repeat(address: &str, chord: &Chord, reps: usize) -> WalkPlan {
    let mut parts = vec![hl::focus_cmd(address)];
    for _ in 0..reps {
        parts.push(hl::shortcut_cmd(chord, address));
    }
    WalkPlan {
        command: parts.join(" && "),
        driver: "office".into(),
    }
}

fn reserved() -> WalkPlan {
    WalkPlan {
        command: "reserved".into(),
        driver: "office".into(),
    }
}
