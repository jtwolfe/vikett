//! Focus a mapped Steam or Heroic window. Play binds `lists.game` and stays
//! reserved: there is no launch command that is not a store URL. The suite
//! fixture carries one title and does not read Steam. Buy is not a page.

use std::collections::BTreeMap;

use crate::classes::client_for_app;
use crate::drivers::hl;
use crate::host::game_id_ok;
use crate::types::{Page, Snap, WalkPlan};

pub fn is_live(page: &Page, snap: &Snap, slots: &BTreeMap<String, String>) -> (bool, String) {
    match page.id.as_str() {
        "games.focus" => focus_live(snap, slots),
        "games.play" => play_live(snap, slots),
        _ => (false, "unknown games page".into()),
    }
}

pub fn fill_walk(page: &Page, slots: &BTreeMap<String, String>, snap: &Snap) -> WalkPlan {
    if page.id == "games.focus" {
        let app = slots.get("app").map(String::as_str).unwrap_or("");
        if let Some(c) = client_for_app(snap, app) {
            return WalkPlan {
                command: hl::focus_cmd(&c.address),
                driver: "games".into(),
            };
        }
    }
    WalkPlan {
        command: "reserved".into(),
        driver: "games".into(),
    }
}

fn focus_live(snap: &Snap, slots: &BTreeMap<String, String>) -> (bool, String) {
    let Some(app) = slots.get("app").map(String::as_str) else {
        return (false, "app missing".into());
    };
    if !matches!(app, "steam" | "heroic") {
        return (false, "not a games app".into());
    }
    match client_for_app(snap, app) {
        Some(c) => (true, format!("client {}", c.class)),
        None => (false, "reserved — game app not mapped".into()),
    }
}

fn play_live(snap: &Snap, slots: &BTreeMap<String, String>) -> (bool, String) {
    let Some(title) = slots.get("title").map(String::as_str) else {
        return (false, "title missing".into());
    };
    if !game_id_ok(title) {
        return (false, "title not a token".into());
    }
    let listed = snap
        .lists
        .get("game")
        .is_some_and(|names| names.iter().any(|n| n == title));
    if !listed {
        return (false, "title not in list".into());
    }
    (false, "reserved — no launch command".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::Catalog;
    use crate::decide::decide;
    use crate::slots;
    use crate::types::{Client, RefereeKind};

    #[test]
    fn play_binds_one_fixture_title_and_does_not_launch() {
        let cat = Catalog::load();
        let snap = cat.snap("desk-rig").unwrap();
        let titles = vec!["celeste".to_string()];
        assert_eq!(snap.lists.get("game"), Some(&titles));
        let play = cat.page("games.play").unwrap();
        let filled = slots::fill(play, "play celeste", snap);
        assert_eq!(
            filled.slots.get("title").map(String::as_str),
            Some("celeste")
        );
        let (ok, why) = is_live(play, snap, &filled.slots);
        assert!(!ok, "{why}");
        assert!(why.contains("reserved"), "{why}");
        let walk = fill_walk(play, &filled.slots, snap).command;
        assert_eq!(walk, "reserved");
        assert!(!walk.contains("celeste"));
        assert!(!walk.contains("steam"));
        assert!(!walk.contains("http"));
        assert!(!walk.contains("rungameid"));

        let other = slots::fill(play, "play hades", snap);
        assert!(!other.slots.contains_key("title"), "{:?}", other.slots);
        let (ok, why) = is_live(play, snap, &other.slots);
        assert!(!ok, "{why}");
        assert!(why.contains("missing"), "{why}");

        let silent = decide(&cat, "play celeste", snap, RefereeKind::Lexical);
        assert!(silent.take.page_id.is_none(), "{:?}", silent.take);
        assert!(silent.walk.is_none());
        let bare = decide(&cat, "play", snap, RefereeKind::Lexical);
        assert_eq!(bare.take.page_id.as_deref(), Some("media.play_pause"));

        let focus = cat.page("games.focus").unwrap();
        let mut unmapped_slots = BTreeMap::new();
        unmapped_slots.insert("app".into(), "steam".into());
        let (ok, why) = is_live(focus, snap, &unmapped_slots);
        assert!(!ok, "{why}");
        assert!(
            why.contains("reserved") || why.contains("not mapped"),
            "{why}"
        );
        let unmapped = decide(&cat, "steam window", snap, RefereeKind::Lexical);
        assert!(unmapped.take.page_id.is_none(), "{:?}", unmapped.take);
        assert!(unmapped.walk.is_none());

        let mut mapped = snap.clone();
        mapped.clients.push(Client {
            address: "0xsteam".into(),
            class: "steam".into(),
            title: "Steam".into(),
            workspace: "1".into(),
            focused: true,
            fullscreen: false,
            floating: false,
        });
        let mut slots = BTreeMap::new();
        slots.insert("app".into(), "steam".into());
        let (ok, why) = is_live(focus, &mapped, &slots);
        assert!(ok, "{why}");
        let cmd = fill_walk(focus, &slots, &mapped).command;
        assert!(cmd.contains("hl.dsp.focus"), "{cmd}");
        assert!(cmd.contains("address:0xsteam"), "{cmd}");
        assert!(!cmd.contains("exec_cmd"), "{cmd}");
        assert!(!cmd.contains("steam://"), "{cmd}");
    }
}
