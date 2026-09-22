//! Notes doors. Daily and vault are private. There is no text slot.
//! Daily is armed only for Logseq (`Alt+J`, journals). Sidebar and vault
//! have no single chord we can send. Focus does not exec.

use std::collections::BTreeMap;

use crate::classes::client_for_app;
use crate::drivers::files::{chord_walk, reserved};
use crate::keymap;
use crate::types::{Page, Snap, WalkPlan};

pub fn is_live(page: &Page, snap: &Snap, slots: &BTreeMap<String, String>) -> (bool, String) {
    let Some(app) = slots.get("app") else {
        return (false, "app missing".into());
    };
    if !is_notes_app(app) {
        return (false, "not a notes app".into());
    }
    match page.id.as_str() {
        "notes.focus" => match client_for_app(snap, app) {
            Some(c) => (true, format!("client {}", c.class)),
            None => (false, "notes app not mapped".into()),
        },
        "notes.vault" | "notes.daily" | "notes.sidebar" => chord_live(page, snap, app, slots),
        _ => (false, "unknown notes page".into()),
    }
}

pub fn fill_walk(page: &Page, slots: &BTreeMap<String, String>, snap: &Snap) -> WalkPlan {
    let app = slots.get("app").map(String::as_str).unwrap_or("");
    if page.id == "notes.focus" {
        return match client_for_app(snap, app) {
            Some(c) => WalkPlan {
                command: crate::drivers::hl::focus_cmd(&c.address),
                driver: "notes".into(),
            },
            None => reserved("notes"),
        };
    }
    chord_walk(&page.id, app, slots, snap, "notes")
}

fn is_notes_app(app: &str) -> bool {
    matches!(app, "obsidian" | "logseq" | "joplin")
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::Catalog;
    use crate::prune;

    #[test]
    fn daily_hidden_from_guest_and_obsidian_has_no_chord() {
        let cat = Catalog::load();
        let page = cat.page("notes.daily").unwrap();
        assert_eq!(page.policy, crate::types::Policy::Private);
        let guest = cat.snap("guest-living").unwrap();
        let mut slots = BTreeMap::new();
        slots.insert("app".into(), "logseq".into());
        let status = prune::is_live(page, guest, &slots);
        assert!(!status.ok);
        assert!(status.why.contains("private"), "{}", status.why);

        let desk = cat.snap("desk-files").unwrap();
        slots.insert("app".into(), "obsidian".into());
        let (ok, why) = is_live(page, desk, &slots);
        assert!(!ok, "{why}");
        assert!(why.contains("chord"), "{why}");
        let plan = fill_walk(page, &slots, desk);
        assert_eq!(plan.command, "reserved");
        assert!(cat.page("notes.vault").unwrap().policy == crate::types::Policy::Private);
    }
}
