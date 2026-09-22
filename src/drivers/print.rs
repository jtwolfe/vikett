//! Print and scan are confirm and reserved. The printer command is not
//! claimed. A copy count is refused before this driver runs.

use std::collections::BTreeMap;

use crate::types::{Page, Snap, WalkPlan};

pub fn is_live(page: &Page, _snap: &Snap, _slots: &BTreeMap<String, String>) -> (bool, String) {
    match page.id.as_str() {
        "print.print" | "print.scan" => (false, "reserved — printer command unknown".into()),
        _ => (false, "unknown print page".into()),
    }
}

pub fn fill_walk(_page: &Page, _slots: &BTreeMap<String, String>, _snap: &Snap) -> WalkPlan {
    WalkPlan {
        command: "reserved".into(),
        driver: "print".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::Catalog;
    use crate::decide::decide;
    use crate::refuse::refused;
    use crate::types::RefereeKind;

    #[test]
    fn print_and_scan_confirm_and_copies_are_refused() {
        let cat = Catalog::load();
        let snap = cat.snap("desk-rig").unwrap();
        for id in ["print.print", "print.scan"] {
            let page = cat.page(id).unwrap();
            assert!(page.confirm, "{id}");
            let (ok, why) = is_live(page, snap, &Default::default());
            assert!(!ok, "{why}");
            assert!(why.contains("reserved"), "{why}");
            let walk = fill_walk(page, &Default::default(), snap).command;
            assert_eq!(walk, "reserved");
            assert!(!walk.contains("lp"));
            assert!(!walk.contains("scanimage"));
            assert!(!walk.contains("copies"));
        }
        let printed = decide(&cat, "print this page", snap, RefereeKind::Lexical);
        assert!(printed.take.page_id.is_none(), "{:?}", printed.take);
        assert!(printed.walk.is_none());
        let scanned = decide(&cat, "scan this page", snap, RefereeKind::Lexical);
        assert!(scanned.take.page_id.is_none(), "{:?}", scanned.take);
        assert!(scanned.walk.is_none());
        assert!(refused("print 3 copies").is_some());
        assert!(refused("scan 2 copies").is_some());
        assert!(refused("print two copies").is_some());
        let copies = decide(&cat, "print 3 copies", snap, RefereeKind::Lexical);
        assert!(copies.take.page_id.is_none(), "{:?}", copies.take);
        assert!(copies.walk.is_none());
    }
}
