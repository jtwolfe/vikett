//! Drawing apps. Save, undo, zoom, and export png send a chord or stay
//! reserved. Export is confirm. Zoom out is Ctrl+minus only where the zoom-in
//! chord exists. Nothing here execs, and a layer or a shape is not a slot.

use std::collections::BTreeMap;

use crate::classes::client_for_app;
use crate::drivers::hl;
use crate::keymap::{self, Chord};
use crate::types::{Page, Snap, WalkPlan};

pub fn is_live(page: &Page, snap: &Snap, slots: &BTreeMap<String, String>) -> (bool, String) {
    let Some(app) = slots.get("app").map(String::as_str) else {
        return (false, "app missing".into());
    };
    if !is_draw_app(app) {
        return (false, "not a drawing app".into());
    }
    match page.id.as_str() {
        "draw.save" | "draw.undo" | "draw.zoom" | "draw.export_png" => {
            chord_live(page, snap, app, slots)
        }
        _ => (false, "unknown draw page".into()),
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
    let reps = if page.id == "draw.zoom" && slots.get("amount").map(String::as_str) == Some("lot") {
        3
    } else {
        1
    };
    repeat(&c.address, &chord, reps)
}

fn is_draw_app(app: &str) -> bool {
    matches!(app, "gimp" | "krita" | "inkscape" | "darktable")
}

fn shaped(page_id: &str, slots: &BTreeMap<String, String>, mut chord: Chord) -> Chord {
    if page_id == "draw.zoom" && slots.get("direction").map(String::as_str) == Some("down") {
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
        driver: "draw".into(),
    }
}

fn reserved() -> WalkPlan {
    WalkPlan {
        command: "reserved".into(),
        driver: "draw".into(),
    }
}

#[cfg(test)]
mod tests {
    use crate::catalog::Catalog;
    use crate::decide::decide;
    use crate::types::RefereeKind;

    #[test]
    fn krita_zoom_notch_and_inkscape_export_confirms() {
        let cat = Catalog::load();
        let snap = cat.snap("desk-edit").unwrap();
        let export = cat.page("draw.export_png").unwrap();
        assert!(export.confirm);
        let lot = decide(
            &cat,
            "larger krita canvas a lot",
            snap,
            RefereeKind::Lexical,
        );
        assert_eq!(lot.take.page_id.as_deref(), Some("draw.zoom"));
        let walk = lot.walk.unwrap().command;
        assert_eq!(walk.matches("key = \"plus\"").count(), 3, "{walk}");
        assert!(walk.contains("hl.dsp.send_shortcut"), "{walk}");
        assert!(!walk.contains("exec_cmd"), "{walk}");
        assert!(!walk.contains("dispatch exec"), "{walk}");

        let png = decide(&cat, "export png from inkscape", snap, RefereeKind::Lexical);
        assert_eq!(png.take.page_id.as_deref(), Some("draw.export_png"));
        assert!(png.confirm);
        let walk = png.walk.unwrap().command;
        assert!(walk.contains("key = \"E\""), "{walk}");
        assert!(walk.contains("address:0xink"), "{walk}");

        let layer = decide(&cat, "select the layer", snap, RefereeKind::Lexical);
        assert!(layer.take.page_id.is_none(), "{:?}", layer.take);
        assert!(layer.walk.is_none());
        let shape = decide(&cat, "draw a shape", snap, RefereeKind::Lexical);
        assert!(shape.take.page_id.is_none(), "{:?}", shape.take);
    }
}
