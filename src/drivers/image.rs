//! Image viewers. Zoom is Ctrl+plus / Ctrl+minus in Loupe, and Shift+plus
//! zoom-in in imv. Next is an unmodified arrow and trash is Delete, so both
//! stay reserved. Trash is still `confirm`. Focus may exec; nothing else may.

use std::collections::BTreeMap;

use crate::classes::{bin_for_app, client_for_app};
use crate::drivers::hl;
use crate::keymap::{self, Chord};
use crate::types::{Page, Snap, WalkPlan};

pub fn is_live(page: &Page, snap: &Snap, slots: &BTreeMap<String, String>) -> (bool, String) {
    let Some(app) = slots.get("app").map(String::as_str) else {
        return (false, "app missing".into());
    };
    if !is_image_app(app) {
        return (false, "not an image viewer".into());
    }
    match page.id.as_str() {
        "image.focus" => focus_live(snap, app),
        "image.next" | "image.zoom" | "image.trash" => chord_live(page, snap, app, slots),
        _ => (false, "unknown image page".into()),
    }
}

pub fn fill_walk(page: &Page, slots: &BTreeMap<String, String>, snap: &Snap) -> WalkPlan {
    let app = slots.get("app").map(String::as_str).unwrap_or("");
    if page.id == "image.focus" {
        return focus_walk(app, snap);
    }
    let Some(c) = client_for_app(snap, app) else {
        return reserved();
    };
    if page.id == "image.zoom" && app == "imv" && down(slots) {
        return reserved();
    }
    let Some(chord) = keymap::chord_for(&page.id, app, &snap.id, slots) else {
        return reserved();
    };
    let chord = shaped(app, slots, chord);
    let reps = if page.id == "image.zoom" && slots.get("amount").map(String::as_str) == Some("lot")
    {
        3
    } else {
        1
    };
    let mut parts = vec![hl::focus_cmd(&c.address)];
    for _ in 0..reps {
        parts.push(hl::shortcut_cmd(&chord, &c.address));
    }
    WalkPlan {
        command: parts.join(" && "),
        driver: "image".into(),
    }
}

fn is_image_app(app: &str) -> bool {
    matches!(app, "loupe" | "imv")
}

fn down(slots: &BTreeMap<String, String>) -> bool {
    slots.get("direction").map(String::as_str) == Some("down")
}

fn shaped(app: &str, slots: &BTreeMap<String, String>, mut chord: Chord) -> Chord {
    if app == "loupe" && down(slots) {
        chord.key = "minus".into();
    }
    chord
}

fn focus_live(snap: &Snap, app: &str) -> (bool, String) {
    if client_for_app(snap, app).is_some() {
        return (true, format!("client {app}"));
    }
    let Some(bin) = bin_for_app(app) else {
        return (false, "image viewer not mapped".into());
    };
    if snap.allowlist.iter().any(|a| a == app) && snap.bins.iter().any(|b| b == bin) {
        (true, format!("exec {bin}"))
    } else {
        (false, "image viewer not mapped".into())
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
    if page.id == "image.zoom" && app == "imv" && down(slots) {
        return (false, "reserved — no chord".into());
    }
    if keymap::chord_for(&page.id, app, &snap.id, slots).is_none() {
        return (false, "reserved — no chord".into());
    }
    (true, format!("client {}", c.class))
}

fn focus_walk(app: &str, snap: &Snap) -> WalkPlan {
    if let Some(c) = client_for_app(snap, app) {
        return WalkPlan {
            command: hl::focus_cmd(&c.address),
            driver: "image".into(),
        };
    }
    let command = bin_for_app(app)
        .filter(|bin| snap.allowlist.iter().any(|a| a == app) && snap.bins.iter().any(|b| b == bin))
        .map(hl::exec_cmd)
        .unwrap_or_else(|| "reserved".into());
    WalkPlan {
        command,
        driver: "image".into(),
    }
}

fn reserved() -> WalkPlan {
    WalkPlan {
        command: "reserved".into(),
        driver: "image".into(),
    }
}

#[cfg(test)]
mod tests {
    use crate::catalog::Catalog;
    use crate::decide::decide;
    use crate::types::RefereeKind;

    #[test]
    fn loupe_zoom_notch_and_trash_confirms_without_walking() {
        let cat = Catalog::load();
        let snap = cat.snap("desk-media").unwrap();
        let trash = cat.page("image.trash").unwrap();
        assert!(trash.confirm);
        let zoom = decide(&cat, "larger loupe photo a lot", snap, RefereeKind::Lexical);
        assert_eq!(zoom.take.page_id.as_deref(), Some("image.zoom"));
        let walk = zoom.walk.unwrap().command;
        assert_eq!(walk.matches("key = \"plus\"").count(), 3, "{walk}");
        assert!(walk.contains("hl.dsp.send_shortcut"), "{walk}");
        assert!(!walk.contains("dispatch exec"), "{walk}");

        let gone = decide(&cat, "trash the loupe picture", snap, RefereeKind::Lexical);
        assert!(gone.take.page_id.is_none(), "{:?}", gone.take);
        assert!(gone.walk.is_none());
        let next = decide(
            &cat,
            "next",
            cat.snap("guest-living").unwrap(),
            RefereeKind::Lexical,
        );
        assert_eq!(next.take.page_id.as_deref(), Some("media.next"));
    }
}
