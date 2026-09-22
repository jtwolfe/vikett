//! Syncthing and KDE Connect stay reserved. This driver does not claim either
//! binary. Send-file is already refused. There is no status field to invent.

use std::collections::BTreeMap;

use crate::types::{Page, Snap, WalkPlan};

pub fn is_live(page: &Page, _snap: &Snap, slots: &BTreeMap<String, String>) -> (bool, String) {
    let Some(app) = slots.get("app").map(String::as_str) else {
        return (false, "app missing".into());
    };
    if !matches!(app, "syncthing" | "kdeconnect") {
        return (false, "not a sync app".into());
    }
    match page.id.as_str() {
        "sync.ask" => (false, "reserved — sync status unknown".into()),
        "sync.ping" => (false, "reserved — sync ping command unknown".into()),
        _ => (false, "unknown sync page".into()),
    }
}

pub fn fill_walk(_page: &Page, _slots: &BTreeMap<String, String>, _snap: &Snap) -> WalkPlan {
    WalkPlan {
        command: "reserved".into(),
        driver: "sync".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::Catalog;
    use crate::decide::decide;
    use crate::types::RefereeKind;

    #[test]
    fn sync_status_and_ping_stay_reserved() {
        let cat = Catalog::load();
        let snap = cat.snap("desk-rig").unwrap();
        let ask = cat.page("sync.ask").unwrap();
        let mut slots = BTreeMap::new();
        slots.insert("app".into(), "syncthing".into());
        let (ok, why) = is_live(ask, snap, &slots);
        assert!(!ok, "{why}");
        assert!(why.contains("reserved"), "{why}");
        let status = decide(&cat, "syncthing status", snap, RefereeKind::Lexical);
        assert!(status.take.page_id.is_none(), "{:?}", status.take);
        assert!(status.answer.is_none());
        assert!(status.walk.is_none());

        let ping = cat.page("sync.ping").unwrap();
        slots.insert("app".into(), "kdeconnect".into());
        let (ok, why) = is_live(ping, snap, &slots);
        assert!(!ok, "{why}");
        assert!(why.contains("reserved"), "{why}");
        let walk = fill_walk(ping, &slots, snap).command;
        assert_eq!(walk, "reserved");
        assert!(!walk.contains("send"));
        assert!(!walk.contains("kdeconnect-cli"));
        let phone = decide(&cat, "ping kdeconnect", snap, RefereeKind::Lexical);
        assert!(phone.take.page_id.is_none(), "{:?}", phone.take);
        assert!(phone.walk.is_none());
    }
}
