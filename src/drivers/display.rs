//! Panel brightness. The v0 notch and night-light strings stay.
//! Per-output brightness is a notch on an authored class (`hdmi` | `edp`)
//! and walks `brightnessctl`. Dead when that class is neither listed nor the
//! classified focused output. No resolution and no refresh.
//! `display.layout` stays reserved: applying `single` or `hdmi-right` needs a
//! mode, and neither `brightnessctl` nor `hl.dsp` can do that without one.

use std::collections::BTreeMap;

use crate::host::output_class;
use crate::types::{Page, Snap, WalkPlan};

pub fn is_live(page: &Page, snap: &Snap, slots: &BTreeMap<String, String>) -> (bool, String) {
    match page.id.as_str() {
        "display.bump" | "display.night" => (true, format!("panel {}", snap.brightness)),
        "display.output" => output_live(snap, slots),
        "display.layout" => layout_live(snap, slots),
        _ => (false, "unknown display page".into()),
    }
}

pub fn fill_walk(page: &Page, slots: &BTreeMap<String, String>, _snap: &Snap) -> WalkPlan {
    let command = match page.id.as_str() {
        "display.bump" => bump_cmd(slots),
        "display.night" => "hyprctl hyprsunset temperature 3500".into(),
        "display.output" => match slots.get("output").map(String::as_str) {
            Some(id) if id == "hdmi" || id == "edp" => {
                let (step, dir) = notch(slots);
                format!("brightnessctl --device={id} set {step}{dir}")
            }
            _ => "reserved".into(),
        },
        _ => "reserved".into(),
    };
    WalkPlan {
        command,
        driver: "display".into(),
    }
}

fn bump_cmd(slots: &BTreeMap<String, String>) -> String {
    let (step, dir) = notch(slots);
    format!("brightnessctl set {step}{dir}")
}

fn notch(slots: &BTreeMap<String, String>) -> (&'static str, &'static str) {
    let dir = if slots.get("direction").map(String::as_str) == Some("down") {
        "-"
    } else {
        "+"
    };
    let step = if slots.get("amount").map(String::as_str) == Some("lot") {
        "10%"
    } else {
        "5%"
    };
    (step, dir)
}

fn output_live(snap: &Snap, slots: &BTreeMap<String, String>) -> (bool, String) {
    let Some(id) = slots.get("output") else {
        return (false, "output missing".into());
    };
    if id != "hdmi" && id != "edp" {
        return (false, "output not authored".into());
    }
    if !output_present(snap, id) {
        return (false, "output absent".into());
    }
    (true, format!("output {id}"))
}

fn output_present(snap: &Snap, id: &str) -> bool {
    let listed = snap
        .lists
        .get("output")
        .is_some_and(|names| names.iter().any(|n| n == id));
    let focused = snap
        .focused_output
        .as_deref()
        .and_then(output_class)
        .is_some_and(|class| class == id);
    listed || focused
}

fn layout_live(snap: &Snap, slots: &BTreeMap<String, String>) -> (bool, String) {
    let Some(id) = slots.get("layout") else {
        return (false, "layout missing".into());
    };
    let listed = snap
        .lists
        .get("layout")
        .is_some_and(|names| names.iter().any(|n| n == id));
    if !listed {
        return (false, "layout absent".into());
    }
    (
        false,
        "reserved — no hl.dsp or brightnessctl for a layout preset".into(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::Catalog;
    use crate::decide::decide;
    use crate::prune;
    use crate::types::RefereeKind;

    #[test]
    fn output_notch_is_brightnessctl_and_absent_is_dead() {
        let cat = Catalog::load();
        let desk = cat.snap("desk").unwrap();
        let page = cat.page("display.output").unwrap();
        let mut slots = BTreeMap::new();
        slots.insert("output".into(), "hdmi".into());
        slots.insert("direction".into(), "up".into());
        slots.insert("amount".into(), "little".into());
        let (ok, why) = is_live(page, desk, &slots);
        assert!(ok, "{why}");
        let cmd = fill_walk(page, &slots, desk).command;
        assert!(cmd.contains("brightnessctl --device=hdmi set 5%+"), "{cmd}");
        assert!(!cmd.contains("HDMI-A"), "{cmd}");
        assert!(!cmd.contains("1920"), "{cmd}");
        assert!(!cmd.contains("hz"), "{cmd}");
        assert!(!cmd.contains("focuswindow"), "{cmd}");
        assert!(!cmd.contains("dispatch exec"), "{cmd}");
        slots.insert("direction".into(), "down".into());
        slots.insert("amount".into(), "lot".into());
        let cmd = fill_walk(page, &slots, desk).command;
        assert!(cmd.contains("10%-"), "{cmd}");

        slots.insert("output".into(), "edp".into());
        let (ok, why) = is_live(page, desk, &slots);
        assert!(!ok, "{why}");
        assert!(why.contains("absent"), "{why}");

        let mut bare = desk.clone();
        bare.focused_output = None;
        bare.lists.insert("output".into(), Vec::new());
        slots.insert("output".into(), "hdmi".into());
        let status = prune::is_live(page, &bare, &slots);
        assert!(!status.ok);
        assert!(status.why.contains("absent"), "{}", status.why);

        let r = decide(&cat, "hdmi brighter", desk, RefereeKind::Lexical);
        assert_eq!(r.take.page_id.as_deref(), Some("display.output"));
        assert_eq!(r.take.slots.get("output").map(String::as_str), Some("hdmi"));
        let kitchen = cat.snap("kitchen").unwrap();
        let r = decide(&cat, "hdmi brighter", kitchen, RefereeKind::Lexical);
        assert!(r.take.page_id.is_none(), "{:?}", r.take);
        assert!(r.walk.is_none());

        let bump = decide(&cat, "screen brighter", desk, RefereeKind::Lexical);
        assert_eq!(bump.take.page_id.as_deref(), Some("display.bump"));
        assert!(bump.walk.unwrap().command.contains("brightnessctl set 5%+"));
    }

    #[test]
    fn layout_preset_stays_reserved() {
        let cat = Catalog::load();
        let desk = cat.snap("desk").unwrap();
        let page = cat.page("display.layout").unwrap();
        let mut slots = BTreeMap::new();
        slots.insert("layout".into(), "single".into());
        let (ok, why) = is_live(page, desk, &slots);
        assert!(!ok, "{why}");
        assert!(why.contains("reserved"), "{why}");
        let cmd = fill_walk(page, &slots, desk).command;
        assert_eq!(cmd, "reserved");
        assert!(!cmd.contains("1920") && !cmd.contains('@'));
        slots.insert("layout".into(), "hdmi-right".into());
        let (ok, why) = is_live(page, desk, &slots);
        assert!(!ok, "{why}");
        assert!(why.contains("absent"), "{why}");
        let r = decide(&cat, "hdmi on the right", desk, RefereeKind::Lexical);
        assert!(r.take.page_id.is_none(), "{:?}", r.take);
        assert!(r.walk.is_none());
    }
}
