//! Box start and stop bind `lists.box`. The runtime is not claimed, so both
//! walks stay `reserved`. Stop is confirm on the page. An image is not a name.

use std::collections::BTreeMap;

use crate::types::{Page, Snap, WalkPlan};

pub fn is_live(page: &Page, snap: &Snap, slots: &BTreeMap<String, String>) -> (bool, String) {
    match page.id.as_str() {
        "boxes.start" | "boxes.stop" => name_live(snap, slots),
        _ => (false, "unknown boxes page".into()),
    }
}

pub fn fill_walk(_page: &Page, _slots: &BTreeMap<String, String>, _snap: &Snap) -> WalkPlan {
    WalkPlan {
        command: "reserved".into(),
        driver: "boxes".into(),
    }
}

fn name_live(snap: &Snap, slots: &BTreeMap<String, String>) -> (bool, String) {
    let Some(id) = slots.get("name").map(String::as_str) else {
        return (false, "name missing".into());
    };
    if !listed(snap, id) {
        return (false, "name not in list".into());
    }
    (false, "reserved — box runtime unknown".into())
}

fn listed(snap: &Snap, id: &str) -> bool {
    snap.lists
        .get("box")
        .is_some_and(|names| names.iter().any(|n| n == id))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::Catalog;
    use crate::decide::decide;
    use crate::slots;
    use crate::types::{Policy, RefereeKind};

    #[test]
    fn start_and_stop_use_the_box_list_and_stay_reserved() {
        let cat = Catalog::load();
        let snap = cat.snap("desk-rig").unwrap();
        let names = vec!["arch".to_string(), "fedora".to_string()];
        assert_eq!(snap.lists.get("box"), Some(&names));
        let start = cat.page("boxes.start").unwrap();
        assert!(!start.confirm);
        let stop = cat.page("boxes.stop").unwrap();
        assert!(stop.confirm);
        assert_eq!(stop.policy, Policy::Household);

        let filled = slots::fill(start, "start the arch box", snap);
        assert_eq!(filled.slots.get("name").map(String::as_str), Some("arch"));
        let (ok, why) = is_live(start, snap, &filled.slots);
        assert!(!ok, "{why}");
        assert!(why.contains("reserved"), "{why}");
        let walk = fill_walk(start, &filled.slots, snap).command;
        assert_eq!(walk, "reserved");
        assert!(!walk.contains("arch"));
        assert!(!walk.contains("distrobox"));
        assert!(!walk.contains("podman"));
        assert!(!walk.contains("docker"));
        assert!(!walk.contains("run"));
        assert!(!walk.contains("image"));

        let fedora = slots::fill(start, "start the fedora box", snap);
        assert_eq!(fedora.slots.get("name").map(String::as_str), Some("fedora"));
        let (ok, why) = is_live(stop, snap, &fedora.slots);
        assert!(!ok, "{why}");
        assert!(why.contains("reserved"), "{why}");

        let missing = slots::fill(start, "start the ubuntu box", snap);
        assert!(!missing.slots.contains_key("name"), "{:?}", missing.slots);
        assert!(missing.missing.iter().any(|m| m == "name"));

        let begun = decide(&cat, "start the arch box", snap, RefereeKind::Lexical);
        assert!(begun.take.page_id.is_none(), "{:?}", begun.take);
        assert!(begun.walk.is_none());
        let ended = decide(&cat, "stop the arch box", snap, RefereeKind::Lexical);
        assert!(ended.take.page_id.is_none(), "{:?}", ended.take);
        assert!(ended.walk.is_none());
    }
}
