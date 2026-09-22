//! Reader doors. Next and previous page are Ctrl+Page Down / Up on Evince and
//! Papers (GNOME help). Zoom is Ctrl+plus there and on Foliate. Dark is
//! Zathura recolor, Ctrl+R. Unmodified keys stay reserved. No page number.

use std::collections::BTreeMap;

use crate::classes::client_for_app;
use crate::drivers::files::{chord_walk, reserved};
use crate::keymap;
use crate::types::{Page, Snap, WalkPlan};

pub fn is_live(page: &Page, snap: &Snap, slots: &BTreeMap<String, String>) -> (bool, String) {
    let Some(app) = slots.get("app") else {
        return (false, "app missing".into());
    };
    if !is_reader(app) {
        return (false, "not a reader".into());
    }
    match page.id.as_str() {
        "read.focus" => match client_for_app(snap, app) {
            Some(c) => (true, format!("client {}", c.class)),
            None => (false, "reader not mapped".into()),
        },
        "read.next_page" | "read.prev_page" | "read.chapter_next" | "read.chapter_prev"
        | "read.zoom" | "read.dark" => chord_live(page, snap, app, slots),
        _ => (false, "unknown read page".into()),
    }
}

pub fn fill_walk(page: &Page, slots: &BTreeMap<String, String>, snap: &Snap) -> WalkPlan {
    let app = slots.get("app").map(String::as_str).unwrap_or("");
    if page.id == "read.focus" {
        return match client_for_app(snap, app) {
            Some(c) => WalkPlan {
                command: crate::drivers::hl::focus_cmd(&c.address),
                driver: "read".into(),
            },
            None => reserved("read"),
        };
    }
    chord_walk(&page.id, app, slots, snap, "read")
}

fn is_reader(app: &str) -> bool {
    matches!(app, "zathura" | "evince" | "papers" | "foliate")
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

    #[test]
    fn zoom_lot_repeats_and_chapters_stay_reserved() {
        let cat = Catalog::load();
        let snap = cat.snap("desk-files").unwrap();
        let zoom = cat.page("read.zoom").unwrap();
        let mut slots = BTreeMap::new();
        slots.insert("app".into(), "evince".into());
        slots.insert("direction".into(), "up".into());
        slots.insert("amount".into(), "lot".into());
        let plan = fill_walk(zoom, &slots, snap);
        assert_eq!(
            plan.command.matches("key = \"plus\"").count(),
            3,
            "{}",
            plan.command
        );
        assert!(!plan.command.contains("dispatch exec"), "{}", plan.command);

        let chapter = cat.page("read.chapter_next").unwrap();
        slots.insert("app".into(), "foliate".into());
        let (ok, why) = is_live(chapter, snap, &slots);
        assert!(!ok, "{why}");
        assert_eq!(fill_walk(chapter, &slots, snap).command, "reserved");
    }
}
