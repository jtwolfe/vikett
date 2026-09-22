//! EasyEffects preset and source. Both stay reserved: there is no confirmed
//! chord, and this driver does not invent a plugin graph or `easyeffects -l`.

use std::collections::BTreeMap;

use crate::classes::client_for_app;
use crate::keymap;
use crate::types::{Page, Snap, WalkPlan};

pub fn is_live(page: &Page, snap: &Snap, slots: &BTreeMap<String, String>) -> (bool, String) {
    let Some(app) = slots.get("app").map(String::as_str) else {
        return (false, "app missing".into());
    };
    if app != "easyeffects" {
        return (false, "not easyeffects".into());
    }
    match page.id.as_str() {
        "fx.preset" => enum_live(page, snap, app, slots, "preset", "fx_preset", |id| {
            matches!(id, "default" | "voice")
        }),
        "fx.source" => enum_live(page, snap, app, slots, "source", "source", |id| {
            matches!(id, "mic" | "line")
        }),
        _ => (false, "unknown fx page".into()),
    }
}

pub fn fill_walk(_page: &Page, _slots: &BTreeMap<String, String>, _snap: &Snap) -> WalkPlan {
    // No authored chord, so there is no `hl.dsp.send_shortcut`. Not `easyeffects -l`.
    reserved()
}

fn enum_live(
    page: &Page,
    snap: &Snap,
    app: &str,
    slots: &BTreeMap<String, String>,
    slot: &str,
    list: &str,
    authored: fn(&str) -> bool,
) -> (bool, String) {
    let Some(id) = slots.get(slot).map(String::as_str) else {
        return (false, format!("{slot} missing"));
    };
    if !authored(id) {
        return (false, format!("{slot} not authored"));
    }
    let listed = snap
        .lists
        .get(list)
        .is_some_and(|names| names.iter().any(|n| n == id));
    if !listed {
        return (false, format!("{slot} not in list"));
    }
    if client_for_app(snap, app).is_none() {
        return (false, "no matching client".into());
    }
    if keymap::chord_for(&page.id, app, &snap.id, slots).is_none() {
        return (false, "reserved — no chord".into());
    }
    (true, format!("{slot} {id}"))
}

fn reserved() -> WalkPlan {
    WalkPlan {
        command: "reserved".into(),
        driver: "fx".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::Catalog;
    use crate::decide::decide;
    use crate::types::RefereeKind;

    #[test]
    fn preset_and_source_stay_reserved() {
        let cat = Catalog::load();
        let snap = cat.snap("desk-shelf").unwrap();
        let preset = cat.page("fx.preset").unwrap();
        let mut slots = BTreeMap::new();
        slots.insert("app".into(), "easyeffects".into());
        slots.insert("preset".into(), "voice".into());
        let (ok, why) = is_live(preset, snap, &slots);
        assert!(!ok, "{why}");
        assert!(why.contains("reserved"), "{why}");
        let walk = fill_walk(preset, &slots, snap).command;
        assert_eq!(walk, "reserved");
        assert!(!walk.contains("-l"));
        assert!(!walk.contains("pw-"));
        assert!(!walk.contains("plugin"));
        assert!(!walk.contains("dispatch exec"));

        let source = cat.page("fx.source").unwrap();
        slots.insert("source".into(), "mic".into());
        let (ok, why) = is_live(source, snap, &slots);
        assert!(!ok, "{why}");
        assert!(why.contains("reserved"), "{why}");

        slots.insert("preset".into(), "hall".into());
        let (ok, why) = is_live(preset, snap, &slots);
        assert!(!ok, "{why}");
        assert!(why.contains("not authored"), "{why}");

        let voice = decide(&cat, "easyeffects voice preset", snap, RefereeKind::Lexical);
        assert!(voice.take.page_id.is_none(), "{:?}", voice.take);
        assert!(voice.walk.is_none());
        let mic = decide(&cat, "easyeffects mic source", snap, RefereeKind::Lexical);
        assert!(mic.take.page_id.is_none(), "{:?}", mic.take);
        assert!(mic.walk.is_none());
    }
}
