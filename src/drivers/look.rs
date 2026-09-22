//! Wallpaper is an enum id from `lists.wallpaper`, applied as `swww img <id>`.
//! The id is not a path. `look.pick` reads `picked_hex` and never runs hyprpicker.

use std::collections::BTreeMap;

use serde_json::{json, Value};

use crate::types::{Page, Snap, WalkPlan};

pub fn is_live(page: &Page, snap: &Snap, slots: &BTreeMap<String, String>) -> (bool, String) {
    match page.id.as_str() {
        "look.wallpaper" => wallpaper_live(snap, slots),
        "look.pick" => match hex_field(snap.picked_hex.as_deref()) {
            Some(hex) => (true, format!("hex {hex}")),
            None => (false, "picked hex missing".into()),
        },
        _ => (false, "unknown look page".into()),
    }
}

pub fn fill_walk(page: &Page, slots: &BTreeMap<String, String>, snap: &Snap) -> WalkPlan {
    let command = match page.id.as_str() {
        "look.wallpaper" => wallpaper_cmd(snap, slots).unwrap_or_else(|| "reserved".into()),
        // Ask. Do not exec hyprpicker, and do not apply the hex as a color.
        _ => "reserved".into(),
    };
    WalkPlan {
        command,
        driver: "look".into(),
    }
}

pub fn fill_ask(page: &Page, _slots: &BTreeMap<String, String>, snap: &Snap) -> Value {
    match page.id.as_str() {
        "look.pick" => match hex_field(snap.picked_hex.as_deref()) {
            Some(hex) => json!({ "hex": hex }),
            None => json!({ "unarmed": "look.pick" }),
        },
        _ => json!({ "unarmed": page.id }),
    }
}

fn wallpaper_live(snap: &Snap, slots: &BTreeMap<String, String>) -> (bool, String) {
    let Some(id) = slots.get("wallpaper").map(String::as_str) else {
        return (false, "wallpaper missing".into());
    };
    if !matches!(id, "dark" | "photo") {
        return (false, "wallpaper not authored".into());
    }
    let listed = snap
        .lists
        .get("wallpaper")
        .is_some_and(|names| names.iter().any(|n| n == id));
    if !listed {
        return (false, "wallpaper not in list".into());
    }
    if !snap.bins.iter().any(|b| b == "swww") {
        return (false, "wallpaper tool absent".into());
    }
    (true, format!("wallpaper {id}"))
}

fn wallpaper_cmd(snap: &Snap, slots: &BTreeMap<String, String>) -> Option<String> {
    let (ok, _) = wallpaper_live(snap, slots);
    if !ok {
        return None;
    }
    let id = slots.get("wallpaper").map(String::as_str)?;
    Some(format!("swww img {id}"))
}

fn hex_field(raw: Option<&str>) -> Option<&str> {
    let s = raw?.trim();
    let digits = s.strip_prefix('#').unwrap_or(s);
    if digits.len() == 6 && digits.chars().all(|c| c.is_ascii_hexdigit()) {
        Some(s)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::Catalog;
    use crate::decide::decide;
    use crate::types::RefereeKind;

    #[test]
    fn wallpaper_enum_is_not_a_path_and_picker_does_not_exec() {
        let cat = Catalog::load();
        let snap = cat.snap("desk-shelf").unwrap();
        let dark = decide(&cat, "dark wallpaper", snap, RefereeKind::Lexical);
        assert_eq!(dark.take.page_id.as_deref(), Some("look.wallpaper"));
        assert_eq!(
            dark.take.slots.get("wallpaper").map(String::as_str),
            Some("dark")
        );
        let walk = dark.walk.unwrap().command;
        assert_eq!(walk, "swww img dark");
        assert!(!walk.contains('/'));
        assert!(!walk.contains("hyprpicker"));
        assert!(!walk.contains("dispatch exec"));
        assert!(!walk.contains("hyprctl"));

        let photo = decide(&cat, "photo wallpaper", snap, RefereeKind::Lexical);
        assert_eq!(photo.walk.unwrap().command, "swww img photo");

        let mut bare = snap.clone();
        bare.lists.insert("wallpaper".into(), vec!["dark".into()]);
        let page = cat.page("look.wallpaper").unwrap();
        let mut slots = BTreeMap::new();
        slots.insert("wallpaper".into(), "photo".into());
        let (ok, why) = is_live(page, &bare, &slots);
        assert!(!ok, "{why}");
        assert!(why.contains("not in list"), "{why}");
        let missed = decide(&cat, "blue wallpaper", snap, RefereeKind::Lexical);
        assert!(missed.take.page_id.is_none(), "{:?}", missed.take);
        assert!(missed.walk.is_none());

        let pick = decide(&cat, "pick a color", snap, RefereeKind::Lexical);
        assert_eq!(pick.take.page_id.as_deref(), Some("look.pick"));
        assert!(pick.walk.is_none());
        assert_eq!(pick.answer.unwrap(), json!({ "hex": "#336699" }));
        let reserved = fill_walk(cat.page("look.pick").unwrap(), &Default::default(), snap);
        assert_eq!(reserved.command, "reserved");
        assert!(!reserved.command.contains("hyprpicker"));

        let desk = cat.snap("desk").unwrap();
        let dead = decide(&cat, "pick a color", desk, RefereeKind::Lexical);
        assert!(dead.take.page_id.is_none(), "{:?}", dead.take);
        assert!(dead.walk.is_none());
        assert!(dead.answer.is_none());

        let mut bad = snap.clone();
        bad.picked_hex = Some("red".into());
        let (ok, why) = is_live(cat.page("look.pick").unwrap(), &bad, &Default::default());
        assert!(!ok, "{why}");
        assert!(why.contains("missing"), "{why}");
    }
}
