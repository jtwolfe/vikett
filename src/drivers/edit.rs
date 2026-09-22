//! Editor doors. `edit.focus` is the only exec, and only for an allowlisted
//! project id. The walk never carries the directory path. Save, close, next
//! tab, split, and format send a chord or stay reserved. `codium` windows are
//! class `code` in the class map; this driver still tells the binaries apart.

use std::collections::BTreeMap;

use crate::classes::{app_for_class, bin_for_app};
use crate::drivers::hl;
use crate::keymap;
use crate::types::{Client, Page, Snap, WalkPlan};

pub fn is_live(page: &Page, snap: &Snap, slots: &BTreeMap<String, String>) -> (bool, String) {
    let Some(app) = slots.get("app").map(String::as_str) else {
        return (false, "app missing".into());
    };
    if !is_edit_app(app) {
        return (false, "not an editor".into());
    }
    match page.id.as_str() {
        "edit.focus" => focus_live(snap, app, slots),
        "edit.save" | "edit.close_tab" | "edit.next_tab" | "edit.split" | "edit.format" => {
            chord_live(page, snap, app, slots)
        }
        _ => (false, "unknown edit page".into()),
    }
}

pub fn fill_walk(page: &Page, slots: &BTreeMap<String, String>, snap: &Snap) -> WalkPlan {
    let app = slots.get("app").map(String::as_str).unwrap_or("");
    if page.id == "edit.focus" {
        return focus_walk(app, slots, snap);
    }
    let Some(c) = edit_client(snap, app) else {
        return reserved();
    };
    let Some(chord) = keymap::chord_for(&page.id, app, &snap.id, slots) else {
        return reserved();
    };
    repeat(&c.address, &chord, 1)
}

fn is_edit_app(app: &str) -> bool {
    matches!(app, "nvim" | "helix" | "code" | "codium" | "zed" | "emacs")
}

fn listed(snap: &Snap, id: &str) -> bool {
    snap.lists
        .get("project")
        .is_some_and(|names| !names.is_empty() && names.iter().any(|n| n == id))
}

/// `class_to_app("codium")` stays `code`. A codium door still needs the
/// codium window, and a code door must not grab it.
fn edit_client<'a>(snap: &'a Snap, app: &str) -> Option<&'a Client> {
    let mut found = None;
    for c in &snap.clients {
        let class = c.class.to_lowercase();
        let is_codium = class.contains("codium");
        let hit = match app {
            "codium" => is_codium,
            "code" => app_for_class(&c.class) == Some("code") && !is_codium,
            other => app_for_class(&c.class) == Some(other),
        };
        if !hit {
            continue;
        }
        if c.focused {
            return Some(c);
        }
        if found.is_none() {
            found = Some(c);
        }
    }
    found
}

fn focus_live(snap: &Snap, app: &str, slots: &BTreeMap<String, String>) -> (bool, String) {
    let Some(project) = slots.get("project").map(String::as_str) else {
        return (false, "project missing".into());
    };
    if !listed(snap, project) {
        return (false, "project not in list".into());
    }
    if edit_client(snap, app).is_some() {
        return (true, format!("client {app}"));
    }
    let Some(bin) = bin_for_app(app) else {
        return (false, "editor not mapped".into());
    };
    if snap.allowlist.iter().any(|a| a == app) && snap.bins.iter().any(|b| b == bin) {
        (true, format!("exec {bin}"))
    } else {
        (false, "editor not mapped".into())
    }
}

fn chord_live(
    page: &Page,
    snap: &Snap,
    app: &str,
    slots: &BTreeMap<String, String>,
) -> (bool, String) {
    let Some(c) = edit_client(snap, app) else {
        return (false, "no matching client".into());
    };
    if keymap::chord_for(&page.id, app, &snap.id, slots).is_none() {
        return (false, "reserved — no chord".into());
    }
    (true, format!("client {}", c.class))
}

fn focus_walk(app: &str, slots: &BTreeMap<String, String>, snap: &Snap) -> WalkPlan {
    let project = slots.get("project").map(String::as_str).unwrap_or("");
    if !listed(snap, project) {
        return reserved();
    }
    if let Some(c) = edit_client(snap, app) {
        return WalkPlan {
            command: hl::focus_cmd(&c.address),
            driver: "edit".into(),
        };
    }
    let command = bin_for_app(app)
        .filter(|bin| snap.allowlist.iter().any(|a| a == app) && snap.bins.iter().any(|b| b == bin))
        .map(hl::exec_cmd)
        .unwrap_or_else(|| "reserved".into());
    WalkPlan {
        command,
        driver: "edit".into(),
    }
}

fn repeat(address: &str, chord: &crate::keymap::Chord, reps: usize) -> WalkPlan {
    let mut parts = vec![hl::focus_cmd(address)];
    for _ in 0..reps {
        parts.push(hl::shortcut_cmd(chord, address));
    }
    WalkPlan {
        command: parts.join(" && "),
        driver: "edit".into(),
    }
}

fn reserved() -> WalkPlan {
    WalkPlan {
        command: "reserved".into(),
        driver: "edit".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::Catalog;
    use crate::decide::decide;
    use crate::types::RefereeKind;

    #[test]
    fn focus_project_does_not_pass_a_path_or_steal_next_tab() {
        let cat = Catalog::load();
        let snap = cat.snap("desk-edit").unwrap();
        let page = cat.page("edit.focus").unwrap();
        let mut slots = BTreeMap::new();
        slots.insert("app".into(), "nvim".into());
        slots.insert("project".into(), "vikett".into());
        let plan = fill_walk(page, &slots, snap);
        assert!(plan.command.contains("hl.dsp.focus"), "{}", plan.command);
        assert!(plan.command.contains("address:0xnvim"), "{}", plan.command);
        assert!(!plan.command.contains("exec_cmd"), "{}", plan.command);
        assert!(!plan.command.contains("vikett"), "{}", plan.command);
        assert!(!plan.command.contains('/'), "{}", plan.command);

        slots.insert("app".into(), "helix".into());
        let exec = fill_walk(page, &slots, snap);
        assert!(
            exec.command.contains("exec_cmd(\"hx\")"),
            "{}",
            exec.command
        );
        assert!(!exec.command.contains("vikett"), "{}", exec.command);
        assert!(!exec.command.contains("dispatch exec"), "{}", exec.command);

        slots.insert("app".into(), "emacs".into());
        let (ok, _) = is_live(page, snap, &slots);
        assert!(!ok);
        let bare = fill_walk(page, &slots, snap);
        assert_eq!(bare.command, "reserved");

        let zen = decide(
            &cat,
            "next tab",
            cat.snap("desk-zen").unwrap(),
            RefereeKind::Lexical,
        );
        assert_eq!(zen.take.page_id.as_deref(), Some("browser.tab_next"));
        let editor = decide(&cat, "next tab", snap, RefereeKind::Lexical);
        assert!(editor.take.page_id.is_none(), "{:?}", editor.take);
        let named = decide(&cat, "next editor tab", snap, RefereeKind::Lexical);
        assert_eq!(named.take.page_id.as_deref(), Some("edit.next_tab"));
        let guest = decide(
            &cat,
            "next editor tab",
            cat.snap("guest-living").unwrap(),
            RefereeKind::Lexical,
        );
        assert!(guest.take.page_id.is_none(), "{:?}", guest.take);
        assert_ne!(guest.take.page_id.as_deref(), Some("media.next"));
    }
}
