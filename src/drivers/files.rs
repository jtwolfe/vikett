//! File-manager doors. Paths are an enum in this file, never the utterance.
//! Focus is `hl.dsp.focus` only when that class is mapped. Open may `exec_cmd`
//! a binary plus an authored path. Trash has no armable chord (Delete is
//! unmodified; empty mods are only legal for F11) and stays reserved.

use std::collections::BTreeMap;

use crate::classes::{bin_for_app, client_for_app};
use crate::drivers::hl;
use crate::keymap;
use crate::types::{Page, Snap, WalkPlan};

pub fn is_live(page: &Page, snap: &Snap, slots: &BTreeMap<String, String>) -> (bool, String) {
    let Some(app) = slots.get("app") else {
        return (false, "app missing".into());
    };
    if !is_files_app(app) {
        return (false, "not a file manager".into());
    }
    match page.id.as_str() {
        "files.focus" => mapped(snap, app),
        "files.open" => open_live(snap, app, slots),
        "files.back" | "files.up" | "files.hidden" | "files.sort" | "files.trash" => {
            chord_live(page, snap, app, slots)
        }
        _ => (false, "unknown files page".into()),
    }
}

pub fn fill_walk(page: &Page, slots: &BTreeMap<String, String>, snap: &Snap) -> WalkPlan {
    let app = slots.get("app").map(String::as_str).unwrap_or("");
    let driver = "files";
    if page.id == "files.focus" {
        return focus_walk(app, snap, driver);
    }
    if page.id == "files.open" {
        return open_walk(app, slots, snap, driver);
    }
    chord_walk(&page.id, app, slots, snap, driver)
}

fn is_files_app(app: &str) -> bool {
    matches!(app, "nautilus" | "nemo" | "thunar" | "dolphin" | "yazi")
}

/// Authored directories. The utterance only ever carries the id.
pub fn authored_dir(id: &str) -> Option<&'static str> {
    match id {
        "home" => Some("/home/jim"),
        "downloads" => Some("/home/jim/Downloads"),
        "pictures" => Some("/home/jim/Pictures"),
        "documents" => Some("/home/jim/Documents"),
        _ => None,
    }
}

fn listed(snap: &Snap, id: &str) -> bool {
    snap.lists
        .get("dir")
        .is_some_and(|names| !names.is_empty() && names.iter().any(|n| n == id))
}

fn mapped(snap: &Snap, app: &str) -> (bool, String) {
    match client_for_app(snap, app) {
        Some(c) => (true, format!("client {}", c.class)),
        None => (false, "file manager not mapped".into()),
    }
}

fn open_live(snap: &Snap, app: &str, slots: &BTreeMap<String, String>) -> (bool, String) {
    let Some(dir) = slots.get("dir") else {
        return (false, "dir missing".into());
    };
    if authored_dir(dir).is_none() {
        return (false, "dir not authored".into());
    }
    if !listed(snap, dir) {
        return (false, "dir not in list".into());
    }
    if client_for_app(snap, app).is_some() {
        return (true, format!("client {app}"));
    }
    let Some(bin) = bin_for_app(app) else {
        return (false, "file manager not mapped".into());
    };
    if snap.allowlist.iter().any(|a| a == app) && snap.bins.iter().any(|b| b == bin) {
        (true, format!("exec {bin}"))
    } else {
        (false, "file manager not mapped".into())
    }
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

fn focus_walk(app: &str, snap: &Snap, driver: &str) -> WalkPlan {
    match client_for_app(snap, app) {
        Some(c) => WalkPlan {
            command: hl::focus_cmd(&c.address),
            driver: driver.into(),
        },
        None => reserved(driver),
    }
}

fn open_walk(app: &str, slots: &BTreeMap<String, String>, snap: &Snap, driver: &str) -> WalkPlan {
    let dir = slots.get("dir").map(String::as_str).unwrap_or("");
    let Some(path) = authored_dir(dir) else {
        return reserved(driver);
    };
    let Some(bin) = bin_for_app(app) else {
        return reserved(driver);
    };
    let exec = hl::exec_cmd(&format!("{bin} {path}"));
    let command = match client_for_app(snap, app) {
        Some(c) => format!("{} && {exec}", hl::focus_cmd(&c.address)),
        None => exec,
    };
    WalkPlan {
        command,
        driver: driver.into(),
    }
}

pub(crate) fn chord_walk(
    page_id: &str,
    app: &str,
    slots: &BTreeMap<String, String>,
    snap: &Snap,
    driver: &str,
) -> WalkPlan {
    let Some(c) = client_for_app(snap, app) else {
        return reserved(driver);
    };
    let Some(mut chord) = keymap::chord_for(page_id, app, &snap.id, slots) else {
        return reserved(driver);
    };
    let mut reps = 1;
    if page_id == "read.zoom" {
        if slots.get("direction").map(String::as_str) == Some("down") {
            chord.key = "minus".into();
        }
        if slots.get("amount").map(String::as_str) == Some("lot") {
            reps = 3;
        }
    }
    let mut parts = vec![hl::focus_cmd(&c.address)];
    for _ in 0..reps {
        parts.push(hl::shortcut_cmd(&chord, &c.address));
    }
    WalkPlan {
        command: parts.join(" && "),
        driver: driver.into(),
    }
}

pub(crate) fn reserved(driver: &str) -> WalkPlan {
    WalkPlan {
        command: "reserved".into(),
        driver: driver.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::Catalog;

    #[test]
    fn open_uses_authored_path_not_the_utterance() {
        let cat = Catalog::load();
        let snap = cat.snap("desk-files").unwrap();
        let page = cat.page("files.open").unwrap();
        let mut slots = BTreeMap::new();
        slots.insert("app".into(), "nautilus".into());
        slots.insert("dir".into(), "downloads".into());
        let plan = fill_walk(page, &slots, snap);
        assert!(
            plan.command
                .contains("exec_cmd(\"nautilus /home/jim/Downloads\")"),
            "{}",
            plan.command
        );
        assert!(plan.command.contains("hl.dsp.focus"), "{}", plan.command);
        assert!(!plan.command.contains("dispatch exec"), "{}", plan.command);
        assert!(!plan.command.contains(".."), "{}", plan.command);
        assert!(authored_dir("../etc").is_none());
        assert!(authored_dir("/tmp").is_none());
    }

    #[test]
    fn unmapped_open_execs_and_trash_stays_reserved() {
        let cat = Catalog::load();
        let snap = cat.snap("desk-files").unwrap();
        let open = cat.page("files.open").unwrap();
        let mut slots = BTreeMap::new();
        slots.insert("app".into(), "yazi".into());
        slots.insert("dir".into(), "documents".into());
        let (ok, why) = is_live(open, snap, &slots);
        assert!(ok, "{why}");
        let plan = fill_walk(open, &slots, snap);
        assert!(
            plan.command
                .contains("exec_cmd(\"yazi /home/jim/Documents\")"),
            "{}",
            plan.command
        );
        assert!(!plan.command.contains("hl.dsp.focus"), "{}", plan.command);
        assert!(!plan.command.contains("dispatch exec"), "{}", plan.command);

        let trash = cat.page("files.trash").unwrap();
        assert!(trash.confirm);
        slots.insert("app".into(), "nautilus".into());
        let (ok, why) = is_live(trash, snap, &slots);
        assert!(!ok, "{why}");
        let plan = fill_walk(trash, &slots, snap);
        assert_eq!(plan.command, "reserved");
    }

    #[test]
    fn open_downloads_on_desk_stays_launch() {
        let cat = Catalog::load();
        let snap = cat.snap("desk").unwrap();
        let result = crate::decide(
            &cat,
            "open downloads",
            snap,
            crate::types::RefereeKind::Lexical,
        );
        assert_eq!(result.take.page_id.as_deref(), Some("launch.app"));
        assert_eq!(
            result.take.slots.get("app").map(String::as_str),
            Some("files")
        );
    }

    #[test]
    fn empty_dir_list_kills_open() {
        let cat = Catalog::load();
        let mut snap = cat.snap("desk-files").unwrap().clone();
        snap.lists.insert("dir".into(), Vec::new());
        let page = cat.page("files.open").unwrap();
        let mut slots = BTreeMap::new();
        slots.insert("app".into(), "nautilus".into());
        slots.insert("dir".into(), "home".into());
        let (ok, why) = is_live(page, &snap, &slots);
        assert!(!ok, "{why}");
        assert!(why.contains("list"), "{why}");
    }
}
