//! OBS record is confirm and reserved: the record command is not claimed.
//! Scene is `lists.obs_scene` (`desk` or `cam`) and reserved until a chord
//! exists. Stream is not a page.

use std::collections::BTreeMap;

use crate::types::{Page, Snap, WalkPlan};

pub fn is_live(page: &Page, snap: &Snap, slots: &BTreeMap<String, String>) -> (bool, String) {
    let Some(app) = slots.get("app").map(String::as_str) else {
        return (false, "app missing".into());
    };
    if app != "obs" {
        return (false, "not obs".into());
    }
    match page.id.as_str() {
        "obs.record" => (false, "reserved — obs record command unknown".into()),
        "obs.scene" => scene_live(snap, slots),
        _ => (false, "unknown obs page".into()),
    }
}

pub fn fill_walk(_page: &Page, _slots: &BTreeMap<String, String>, _snap: &Snap) -> WalkPlan {
    WalkPlan {
        command: "reserved".into(),
        driver: "obs".into(),
    }
}

fn scene_live(snap: &Snap, slots: &BTreeMap<String, String>) -> (bool, String) {
    let Some(id) = slots.get("scene").map(String::as_str) else {
        return (false, "scene missing".into());
    };
    if !matches!(id, "desk" | "cam") {
        return (false, "scene not authored".into());
    }
    let listed = snap
        .lists
        .get("obs_scene")
        .is_some_and(|names| names.iter().any(|n| n == id));
    if !listed {
        return (false, "scene not in list".into());
    }
    (false, "reserved — no chord".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::Catalog;
    use crate::decide::decide;
    use crate::refuse::refused;
    use crate::slots;
    use crate::types::RefereeKind;

    #[test]
    fn record_confirms_and_scene_is_the_enum() {
        let cat = Catalog::load();
        let snap = cat.snap("desk-rig").unwrap();
        let record = cat.page("obs.record").unwrap();
        assert!(record.confirm);
        let mut slots = BTreeMap::new();
        slots.insert("app".into(), "obs".into());
        let (ok, why) = is_live(record, snap, &slots);
        assert!(!ok, "{why}");
        assert!(why.contains("reserved"), "{why}");
        let walk = fill_walk(record, &slots, snap).command;
        assert_eq!(walk, "reserved");
        assert!(!walk.contains("obs"));
        assert!(!walk.contains("wf-recorder"));
        assert!(!walk.contains("stream"));

        let scene = cat.page("obs.scene").unwrap();
        let filled = slots::fill(scene, "obs scene desk", snap);
        assert_eq!(filled.slots.get("scene").map(String::as_str), Some("desk"));
        assert_eq!(filled.slots.get("app").map(String::as_str), Some("obs"));
        let (ok, why) = is_live(scene, snap, &filled.slots);
        assert!(!ok, "{why}");
        assert!(why.contains("reserved"), "{why}");
        let cam = slots::fill(scene, "obs scene cam", snap);
        assert_eq!(cam.slots.get("scene").map(String::as_str), Some("cam"));
        let stage = slots::fill(scene, "obs scene stage", snap);
        assert!(!stage.slots.contains_key("scene"), "{:?}", stage.slots);

        let shelf = cat.snap("desk-shelf").unwrap();
        let obs_rec = decide(&cat, "start obs recording", shelf, RefereeKind::Lexical);
        assert!(obs_rec.take.page_id.is_none(), "{:?}", obs_rec.take);
        assert!(obs_rec.walk.is_none());
        let cap = decide(&cat, "start recording", shelf, RefereeKind::Lexical);
        assert_eq!(cap.take.page_id.as_deref(), Some("capture.record_start"));
        assert!(refused("start streaming").is_some());
        assert!(refused("start streaming the screen").is_some());
        let stream = decide(&cat, "start streaming", shelf, RefereeKind::Lexical);
        assert!(stream.take.page_id.is_none(), "{:?}", stream.take);
        assert!(stream.walk.is_none());
    }
}
